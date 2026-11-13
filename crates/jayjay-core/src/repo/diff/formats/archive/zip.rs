use crate::types::*;

use super::super::types::projection_error;
use super::listing::ArchiveEntry;

const END_RECORD: &[u8] = b"PK\x05\x06";
const END_RECORD_LEN: usize = 22;
const ZIP64_LOCATOR: &[u8] = b"PK\x06\x07";
const ZIP64_LOCATOR_LEN: usize = 20;
const ZIP64_END_RECORD: &[u8] = b"PK\x06\x06";
const ZIP64_END_RECORD_LEN: usize = 56;
const ENTRY: &[u8] = b"PK\x01\x02";
const ENTRY_LEN: usize = 46;
const ZIP64_EXTRA_FIELD: usize = 1;

#[derive(Clone, Copy)]
pub(super) struct ZipReader<'a>(&'a [u8]);

impl<'a> ZipReader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    pub(super) fn is_archive(self) -> bool {
        self.central_directory().is_some()
    }

    pub(super) fn entries(self) -> JayResult<Vec<ArchiveEntry>> {
        let mut directory = self
            .central_directory()
            .ok_or_else(|| projection_error("not a zip archive"))?;
        let mut entries = Vec::new();
        while !directory.0.is_empty() {
            let (entry, len) = directory
                .entry()
                .ok_or_else(|| projection_error("zip central directory is corrupt"))?;
            entries.push(entry);
            directory = Self(&directory.0[len..]);
        }
        Ok(entries)
    }

    /// Found backwards from the end record, not by its stored offset, so a jar with a launcher script prepended still lists.
    fn central_directory(self) -> Option<Self> {
        let last = self.0.len().checked_sub(END_RECORD_LEN)?;
        let end_record = (last.saturating_sub(usize::from(u16::MAX))..=last)
            .rev()
            .find(|&at| {
                self.has_signature(at, END_RECORD)
                    && self
                        .u16_at(at + 20)
                        .is_some_and(|comment| at + END_RECORD_LEN + comment == self.0.len())
            })?;
        let (end, size) = match self.zip64_end_record(end_record) {
            Some(record) => (record, self.u64_at(record + 40)?),
            None => (end_record, u64::from(self.u32_at(end_record + 12)?)),
        };
        let start = end.checked_sub(usize::try_from(size).ok()?)?;
        self.0.get(start..end).map(Self)
    }

    /// The stored offset is tried first; the fixed-size position covers an archive with data prepended, whose offsets are all shifted.
    fn zip64_end_record(self, end_record: usize) -> Option<usize> {
        let locator = end_record.checked_sub(ZIP64_LOCATOR_LEN)?;
        if !self.has_signature(locator, ZIP64_LOCATOR) {
            return None;
        }
        let stored = usize::try_from(self.u64_at(locator + 8)?).ok();
        [stored, locator.checked_sub(ZIP64_END_RECORD_LEN)]
            .into_iter()
            .flatten()
            .find(|&at| {
                self.has_signature(at, ZIP64_END_RECORD)
                    && self
                        .u64_at(at + 4)
                        .and_then(|len| len.checked_add(at as u64 + 12))
                        == Some(locator as u64)
            })
    }

    fn entry(self) -> Option<(ArchiveEntry, usize)> {
        if !self.has_signature(0, ENTRY) {
            return None;
        }
        let name_end = ENTRY_LEN + self.u16_at(28)?;
        let extra_end = name_end + self.u16_at(30)?;
        let len = extra_end + self.u16_at(32)?;
        let extra = Self(self.0.get(name_end..extra_end)?);
        let size = self.u32_at(24)?;
        let size = (size == u32::MAX)
            .then(|| extra.zip64_size())
            .flatten()
            .unwrap_or(u64::from(size));
        let entry = ArchiveEntry {
            name: self.0[ENTRY_LEN..name_end].to_vec(),
            size,
            crc32: self.u32_at(16)?,
        };
        (len <= self.0.len()).then_some((entry, len))
    }

    fn zip64_size(mut self) -> Option<u64> {
        loop {
            let field = Self(self.0.get(4..4 + self.u16_at(2)?)?);
            if self.u16_at(0)? == ZIP64_EXTRA_FIELD {
                return field.u64_at(0);
            }
            self = Self(&self.0[4 + field.0.len()..]);
        }
    }

    fn has_signature(self, at: usize, signature: &[u8]) -> bool {
        self.0
            .get(at..)
            .is_some_and(|rest| rest.starts_with(signature))
    }

    fn read<const N: usize>(self, at: usize) -> Option<[u8; N]> {
        self.0.get(at..)?.first_chunk().copied()
    }

    fn u16_at(self, at: usize) -> Option<usize> {
        self.read(at)
            .map(|raw| usize::from(u16::from_le_bytes(raw)))
    }

    fn u32_at(self, at: usize) -> Option<u32> {
        self.read(at).map(u32::from_le_bytes)
    }

    fn u64_at(self, at: usize) -> Option<u64> {
        self.read(at).map(u64::from_le_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::ZipReader;

    fn entry(name: &[u8], crc: u32, size: u32, extra: &[u8]) -> Vec<u8> {
        let mut entry = b"PK\x01\x02".to_vec();
        entry.resize(46, 0);
        entry[16..20].copy_from_slice(&crc.to_le_bytes());
        entry[24..28].copy_from_slice(&size.to_le_bytes());
        entry[28..30].copy_from_slice(&(name.len() as u16).to_le_bytes());
        entry[30..32].copy_from_slice(&(extra.len() as u16).to_le_bytes());
        entry.extend_from_slice(name);
        entry.extend_from_slice(extra);
        entry
    }

    fn end_record(directory_len: u32, comment: &[u8]) -> Vec<u8> {
        let mut record = b"PK\x05\x06".to_vec();
        record.resize(22, 0);
        record[12..16].copy_from_slice(&directory_len.to_le_bytes());
        record[20..22].copy_from_slice(&(comment.len() as u16).to_le_bytes());
        record.extend_from_slice(comment);
        record
    }

    fn zip64_end(directory_len: usize, stored_offset: usize, extensible: &[u8]) -> Vec<u8> {
        let mut record = b"PK\x06\x06".to_vec();
        record.resize(56, 0);
        record[4..12].copy_from_slice(&(44 + extensible.len() as u64).to_le_bytes());
        record[40..48].copy_from_slice(&(directory_len as u64).to_le_bytes());
        let mut locator = b"PK\x06\x07".to_vec();
        locator.resize(20, 0);
        locator[8..16].copy_from_slice(&(stored_offset as u64).to_le_bytes());
        [record, extensible.to_vec(), locator].concat()
    }

    #[test]
    fn reads_zip64_entries_past_prepended_data_and_decoy_end_records() {
        let mut zip64_extra = vec![0xCA, 0xFE, 2, 0, 9, 9, 1, 0, 8, 0];
        zip64_extra.extend_from_slice(&5_000_000_000u64.to_le_bytes());
        let directory = [
            entry(b"big.bin", 0xDEADBEEF, u32::MAX, &zip64_extra),
            entry(b"small.txt", 1, 2, &[]),
        ]
        .concat();
        let decoy_comment = [end_record(0, &[]), b"suffix".to_vec()].concat();
        let decoy_record = zip64_end(0, 0, &[])[..56].to_vec();
        let prepended = [
            decoy_record.clone(),
            b"#!/bin/sh\nexec java -jar \"$0\"\n".to_vec(),
            directory.clone(),
            zip64_end(directory.len(), 0, &[]),
            end_record(u32::MAX, &decoy_comment),
        ]
        .concat();
        let extensible = [
            directory.clone(),
            zip64_end(directory.len(), directory.len(), &decoy_record),
            end_record(u32::MAX, &[]),
        ]
        .concat();

        for archive in [prepended, extensible] {
            let entries = ZipReader::new(&archive).entries().expect("read entries");
            let read: Vec<_> = entries
                .iter()
                .map(|entry| (entry.name.as_slice(), entry.size, entry.crc32))
                .collect();
            assert_eq!(
                read,
                [
                    (b"big.bin".as_slice(), 5_000_000_000, 0xDEADBEEF),
                    (b"small.txt".as_slice(), 2, 1)
                ]
            );
        }
    }

    #[test]
    fn rejects_a_directory_that_ends_mid_entry() {
        let mut directory = entry(b"a.txt", 1, 2, &[]);
        directory.truncate(40);
        let archive = [directory.clone(), end_record(directory.len() as u32, &[])].concat();

        let error = ZipReader::new(&archive)
            .entries()
            .err()
            .expect("corrupt directory");
        assert!(error.to_string().contains("corrupt"), "{error}");
    }
}

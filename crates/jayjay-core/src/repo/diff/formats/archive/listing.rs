use std::fmt::{self, Display, Formatter, Write as _};

pub(super) struct ArchiveEntry {
    pub(super) name: Vec<u8>,
    pub(super) size: u64,
    pub(super) crc32: u32,
}

/// Fixed-width columns come before the path so a changed entry is a one-line diff.
pub(super) struct ArchiveListing(Vec<ArchiveEntry>);

impl ArchiveListing {
    pub(super) fn new(mut entries: Vec<ArchiveEntry>) -> Self {
        entries.retain(|entry| !entry.name.ends_with(b"/"));
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Self(entries)
    }
}

impl Display for ArchiveListing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for entry in &self.0 {
            write!(f, "{:08x}  {:>10}  ", entry.crc32, entry.size)?;
            for chunk in entry.name.utf8_chunks() {
                for ch in chunk.valid().chars() {
                    if ch.is_control() || ch == '\\' {
                        write!(f, "{}", ch.escape_default())?;
                    } else {
                        f.write_char(ch)?;
                    }
                }
                for byte in chunk.invalid() {
                    write!(f, "\\x{byte:02x}")?;
                }
            }
            f.write_char('\n')?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ArchiveEntry, ArchiveListing};

    #[test]
    fn lists_files_by_path_with_distinct_names_kept_distinct() {
        let entry = |name: &[u8], size, crc32| ArchiveEntry {
            name: name.to_vec(),
            size,
            crc32,
        };
        let listing = ArchiveListing::new(vec![
            entry(b"lib/", 0, 0),
            entry(b"lib/b\nc.txt", 5_000_000_000, 0xDEADBEEF),
            entry(b"lib/b\\nc.txt", 2, 2),
            entry(b"lib/\x82.txt", 3, 3),
            entry(b"META-INF/MANIFEST.MF", 88, 1),
        ]);

        assert_eq!(
            listing.to_string(),
            "00000001          88  META-INF/MANIFEST.MF\ndeadbeef  5000000000  lib/b\\nc.txt\n00000002           2  lib/b\\\\nc.txt\n00000003           3  lib/\\x82.txt\n"
        );
    }
}

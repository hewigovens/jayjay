use icu_decimal::{DecimalFormatter, input::Decimal};
use icu_locale_core::Locale;

use crate::diff::{DiffSpan, DiffSpanStyle};
use crate::placeholder::binary_file_size;
use crate::syntax::SyntaxToken;

pub struct FileSizeFormatter {
    decimal: DecimalFormatter,
}

impl FileSizeFormatter {
    pub fn new(locale: &str) -> Self {
        let (identifier, keywords) = locale.split_once('@').unwrap_or((locale, ""));
        let mut identifier = identifier
            .split('.')
            .next()
            .unwrap_or_default()
            .replace('_', "-");
        if let Some(numbering) = keywords
            .split(';')
            .find_map(|keyword| keyword.strip_prefix("numbers="))
        {
            identifier.push_str("-u-nu-");
            identifier.push_str(numbering);
        }
        let locale = identifier.parse::<Locale>().unwrap_or(Locale::UNKNOWN);
        let decimal = DecimalFormatter::try_new(locale.into(), Default::default())
            .or_else(|_| DecimalFormatter::try_new(Default::default(), Default::default()))
            .expect("compiled decimal data includes the default locale");
        Self { decimal }
    }

    fn compact(&self, bytes: u64) -> String {
        const UNITS: [&str; 7] = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
        let mut unit = 0;
        let mut divisor = 1_u64;
        while unit < UNITS.len() - 1 && bytes >= divisor * 1_000 {
            unit += 1;
            divisor *= 1_000;
        }
        let mut value = Decimal::from(bytes);
        if unit > 0 {
            value.multiply_pow10(-3 * unit as i16);
            value.round(-1);
            if value.nonzero_magnitude_start() >= 3 && unit < UNITS.len() - 1 {
                unit += 1;
                value.multiply_pow10(-3);
            }
            value.trim_end();
        }
        format!("{} {}", self.decimal.format_to_string(&value), UNITS[unit])
    }

    fn exact(&self, bytes: u64) -> String {
        let count = self.decimal.format_to_string(&Decimal::from(bytes));
        format!("{count} {}", if bytes == 1 { "byte" } else { "bytes" })
    }

    pub fn localize_binary_spans(&self, spans: &mut Vec<DiffSpan>) {
        if !spans.first().is_some_and(|span| span.text.starts_with('<')) {
            return;
        }
        let text: String = spans.iter().map(|span| span.text.as_str()).collect();
        let Some(bytes) = binary_file_size(&text) else {
            return;
        };
        *spans = vec![DiffSpan {
            text: format!("<binary file ({})>", self.compact(bytes)),
            style: DiffSpanStyle::Unchanged,
            token: SyntaxToken::Plain,
            tooltip: Some(self.exact(bytes)),
        }];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::{build_side_by_side_rows, compute_file_diff, wrap_diff_lines, wrap_sbs_rows};

    #[test]
    fn compact_units_round_and_promote_without_losing_exact_bytes() {
        let formatter = FileSizeFormatter::new("en-US");
        for (bytes, compact, exact) in [
            (0, "0 B", "0 bytes"),
            (1, "1 B", "1 byte"),
            (999, "999 B", "999 bytes"),
            (1_000, "1 KB", "1,000 bytes"),
            (141_080, "141.1 KB", "141,080 bytes"),
            (999_949, "999.9 KB", "999,949 bytes"),
            (999_950, "1 MB", "999,950 bytes"),
            (1_234_567, "1.2 MB", "1,234,567 bytes"),
            (1_234_567_890, "1.2 GB", "1,234,567,890 bytes"),
            (1_234_567_890_123, "1.2 TB", "1,234,567,890,123 bytes"),
            (u64::MAX, "18.4 EB", "18,446,744,073,709,551,615 bytes"),
        ] {
            assert_eq!(formatter.compact(bytes), compact);
            assert_eq!(formatter.exact(bytes), exact);
        }
    }

    #[test]
    fn formats_locale_decimal_grouping_and_digits() {
        for (locale, compact, exact) in [
            ("de-DE", "141,1 KB", "141.080 bytes"),
            ("de_DE.UTF-8", "141,1 KB", "141.080 bytes"),
            ("fr-FR", "141,1 KB", "141\u{202f}080 bytes"),
            ("en-IN", "141.1 KB", "1,41,080 bytes"),
            ("ar-EG", "١٤١٫١ KB", "١٤١٬٠٨٠ bytes"),
            ("ar-EG-u-nu-latn", "141.1 KB", "141,080 bytes"),
            ("ar_EG@numbers=latn", "141.1 KB", "141,080 bytes"),
            ("C", "141.1 KB", "141,080 bytes"),
        ] {
            let formatter = FileSizeFormatter::new(locale);
            assert_eq!(formatter.compact(141_080), compact, "{locale}");
            assert_eq!(formatter.exact(141_080), exact, "{locale}");
        }
    }

    #[test]
    fn binary_tooltips_survive_unified_and_side_by_side_wrapping() {
        let original = compute_file_diff(
            "blob.bin",
            "<binary file (141080 bytes)>",
            "<binary file (141081 bytes)>",
            false,
        );
        let mut display = original.clone();
        let formatter = FileSizeFormatter::new("de-DE");
        for line in &mut display.lines {
            formatter.localize_binary_spans(&mut line.spans);
        }
        assert_eq!(original.lines[0].text(), "<binary file (141080 bytes)>");
        assert_eq!(display.lines[0].text(), "<binary file (141,1 KB)>");
        assert_eq!(display.lines[1].text(), "<binary file (141,1 KB)>");
        let unified = wrap_diff_lines(&display.lines, 8);
        for row in &unified {
            let expected = if row.line_ix == 0 {
                "141.080 bytes"
            } else {
                "141.081 bytes"
            };
            assert!(
                row.line
                    .spans
                    .iter()
                    .all(|span| span.tooltip.as_deref() == Some(expected))
            );
        }
        let sbs = wrap_sbs_rows(&build_side_by_side_rows(&display.lines), 8, 8);
        for row in &sbs {
            assert!(
                row.row
                    .old
                    .spans
                    .iter()
                    .all(|span| span.tooltip.as_deref() == Some("141.080 bytes"))
            );
            assert!(
                row.row
                    .new
                    .spans
                    .iter()
                    .all(|span| span.tooltip.as_deref() == Some("141.081 bytes"))
            );
        }
        let mut text = compute_file_diff("plain.txt", "", "let x = 1;", false);
        formatter.localize_binary_spans(&mut text.lines[0].spans);
        assert_eq!(text.lines[0].text(), "let x = 1;");
        assert!(
            text.lines[0]
                .spans
                .iter()
                .all(|span| span.tooltip.is_none())
        );
    }
}

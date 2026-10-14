use jayjay_core::DiffThemeColors;
use jayjay_core::diff::syntax::SyntaxToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SyntaxColors {
    pub(crate) keyword: u32,
    pub(crate) string: u32,
    pub(crate) comment: u32,
    pub(crate) number: u32,
    pub(crate) ty: u32,
}

impl SyntaxColors {
    pub(crate) fn of(diff: &DiffThemeColors) -> Self {
        Self {
            keyword: diff.tok_keyword,
            string: diff.tok_string,
            comment: diff.tok_comment,
            number: diff.tok_number,
            ty: diff.tok_type,
        }
    }

    pub(crate) fn color(&self, token: SyntaxToken) -> Option<u32> {
        match token {
            SyntaxToken::Keyword | SyntaxToken::Operator => Some(self.keyword),
            SyntaxToken::StringLit => Some(self.string),
            SyntaxToken::Comment => Some(self.comment),
            SyntaxToken::Number => Some(self.number),
            SyntaxToken::Type | SyntaxToken::Function | SyntaxToken::Attribute => Some(self.ty),
            SyntaxToken::Plain | SyntaxToken::Variable | SyntaxToken::Punctuation => None,
        }
    }
}

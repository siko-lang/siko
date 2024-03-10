use crate::siko::location::Location::Span;

#[derive(Debug, Clone)]
pub enum LexerError {
    InvalidIdentifier(String, Span),
    UnsupportedCharacter(char, Span),
    UnendingStringLiteral(Span),
}

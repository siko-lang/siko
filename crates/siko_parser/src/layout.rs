use crate::error::LexerError;
use crate::error::LocationInfo;
use crate::token::Token;
use crate::token::TokenInfo;
use crate::token::TokenKind;
use siko_constants::BuiltinOperator;
use siko_location_info::filepath::FilePath;
use siko_location_info::location::Location;
use siko_location_info::span::Span;

struct TokenIterator {
    tokens: Vec<TokenInfo>,
    result: Vec<TokenInfo>,
}

impl TokenIterator {
    fn new(tokens: Vec<TokenInfo>) -> TokenIterator {
        TokenIterator {
            tokens: tokens,
            result: Vec::new(),
        }
    }

    fn is_done(&self) -> bool {
        self.tokens.is_empty()
    }

    fn peek(&self) -> TokenInfo {
        self.tokens.first().expect("ran out of tokeninfo").clone()
    }

    fn advance(&mut self) {
        let t = self.tokens.remove(0);
        self.result.push(t);
    }

    fn add_end(&mut self, token: Token) {
        let location = self.result.last().expect("empty iterator").location.clone();
        self.result.push(TokenInfo {
            token: token,
            location: location,
        });
    }
}

fn process_item(iterator: &mut TokenIterator, block_start: Span) -> bool {
    let mut paren_level = 0;
    let mut end_of_block = false;
    let mut first = true;
    loop {
        if iterator.is_done() {
            end_of_block = true;
            break;
        } else {
            let info = iterator.peek();
            let item_ended = if info.location.span.start <= block_start.start && !first {
                let same = info.location.span.start == block_start.start;
                match info.token.kind() {
                    TokenKind::KeywordThen if same => false,
                    TokenKind::KeywordElse if same => false,
                    _ => true,
                }
            } else {
                false
            };
            first = false;
            if item_ended {
                end_of_block = info.location.span.start < block_start.start;
                break;
            } else {
                match info.token.kind() {
                    TokenKind::KeywordDo | TokenKind::KeywordWhere | TokenKind::KeywordOf => {
                        process_block(iterator);
                    }
                    TokenKind::KeywordModule => {
                        end_of_block = true;
                        break;
                    }
                    TokenKind::LParen => {
                        iterator.advance();
                        paren_level += 1;
                    }
                    TokenKind::RParen => {
                        paren_level -= 1;
                        if paren_level < 0 {
                            end_of_block = true;
                            break;
                        } else {
                            iterator.advance();
                        }
                    }
                    _ => {
                        iterator.advance();
                    }
                }
            }
        }
    }
    iterator.add_end(Token::EndOfItem);
    end_of_block
}

fn process_block(iterator: &mut TokenIterator) {
    iterator.advance();
    if !iterator.is_done() {
        let info = iterator.peek();
        let start_span = info.location.span;
        loop {
            if iterator.is_done() {
                break;
            } else {
                let first = iterator.peek();
                if first.location.span.start < start_span.start {
                    break;
                } else {
                    match first.token.kind() {
                        TokenKind::KeywordDo | TokenKind::KeywordWhere | TokenKind::KeywordOf => {
                            process_block(iterator);
                            iterator.add_end(Token::EndOfItem);
                        }
                        TokenKind::KeywordModule => {
                            break;
                        }
                        _ => {
                            let end_of_block = process_item(iterator, start_span);
                            if end_of_block {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    iterator.add_end(Token::EndOfBlock);
}

fn process_module(iterator: &mut TokenIterator) {
    iterator.advance();
    loop {
        if iterator.is_done() {
            break;
        } else {
            let info = iterator.peek();
            if info.token.kind() == TokenKind::KeywordWhere {
                process_block(iterator);
            } else if info.token.kind() == TokenKind::KeywordModule {
                break;
            } else {
                iterator.advance();
            }
        }
    }
    iterator.add_end(Token::EndOfModule);
}

pub fn process_layout(tokens: Vec<TokenInfo>) -> Vec<TokenInfo> {
    let mut iterator = TokenIterator::new(tokens);
    loop {
        if iterator.is_done() {
            break;
        } else {
            let info = iterator.peek();
            if info.token.kind() == TokenKind::KeywordModule {
                process_module(&mut iterator);
            } else {
                iterator.advance();
            }
        }
    }
    iterator.result
}

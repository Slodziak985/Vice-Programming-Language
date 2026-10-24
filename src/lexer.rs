use crate::error::{ViceError, ViceErrorType};
use crate::token::{Token, TokenKind};

pub struct Lexer {
    source: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, ViceError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                }

                '#' => {
                    self.skip_comment();
                    continue;
                }
                '/' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char('/') {
                        self.skip_comment();
                    } else {
                        tokens.push(Token::at(TokenKind::Slash, "/".into(), line, column, 1));
                    }
                }

                '(' => tokens.push(self.simple(TokenKind::LParen, "(")),
                ')' => tokens.push(self.simple(TokenKind::RParen, ")")),
                '{' => tokens.push(self.simple(TokenKind::LBrace, "{")),
                '}' => tokens.push(self.simple(TokenKind::RBrace, "}")),
                '[' => tokens.push(self.simple(TokenKind::LBracket, "[")),
                ']' => tokens.push(self.simple(TokenKind::RBracket, "]")),
                ',' => tokens.push(self.simple(TokenKind::Comma, ",")),
                ';' => tokens.push(self.simple(TokenKind::Semicolon, ";")),
                ':' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char(':') {
                        tokens.push(Token::at(
                            TokenKind::DoubleColon,
                            "::".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Colon, ":".into(), line, column, 1));
                    }
                }
                '=' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.match_char('>') {
                        tokens.push(Token::at(
                            TokenKind::AssignArrow,
                            "=>".into(),
                            line,
                            column,
                            2,
                        ));
                    } else if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::EqualEqual,
                            "==".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Equal, "=".into(), line, column, 1));
                    }
                }
                '+' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::PlusEqual,
                            "+=".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Plus, "+".into(), line, column, 1));
                    }
                }
                '*' => tokens.push(self.simple(TokenKind::Star, "*")),
                '%' => tokens.push(self.simple(TokenKind::Percent, "%")),
                '?' => tokens.push(self.simple(TokenKind::Question, "?")),

                '<' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.match_char('-') {
                        tokens.push(Token::at(
                            TokenKind::OutputArrow,
                            "<-".into(),
                            line,
                            column,
                            2,
                        ));
                    } else if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::LessEqual,
                            "<=".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Less, "<".into(), line, column, 1));
                    }
                }

                '>' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::GreaterEqual,
                            ">=".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Greater, ">".into(), line, column, 1));
                    }
                }

                '&' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char('&') {
                        tokens.push(Token::at(TokenKind::And, "&&".into(), line, column, 2));
                    } else {
                        tokens.push(Token::at(TokenKind::Ampersand, "&".into(), line, column, 1));
                    }
                }

                '|' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char('|') {
                        tokens.push(Token::at(TokenKind::Or, "||".into(), line, column, 2));
                    } else {
                        tokens.push(Token::at(TokenKind::Pipe, "|".into(), line, column, 1));
                    }
                }

                '!' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::BangEqual,
                            "!=".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Bang, "!".into(), line, column, 1));
                    }
                }

                '-' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();

                    if self.peek() == Some('>') {
                        self.advance();
                        tokens.push(Token::at(
                            TokenKind::TypeArrow,
                            "->".into(),
                            line,
                            column,
                            2,
                        ));
                    } else if self.match_char('=') {
                        tokens.push(Token::at(
                            TokenKind::MinusEqual,
                            "-=".into(),
                            line,
                            column,
                            2,
                        ));
                    } else {
                        tokens.push(Token::at(TokenKind::Minus, "-".into(), line, column, 1));
                    }
                }

                '.' => {
                    let line = self.line;
                    let column = self.column;
                    self.advance();
                    if self.match_char('.') {
                        tokens.push(Token::at(TokenKind::Range, "..".into(), line, column, 2));
                    } else {
                        tokens.push(Token::at(TokenKind::Dot, ".".into(), line, column, 1));
                    }
                }

                '"' => tokens.push(self.string()?),
                '@' => tokens.push(self.attribute()),

                c if c.is_ascii_digit() => tokens.push(self.number()),
                c if Self::is_ident_start(c) => tokens.push(self.identifier()),

                _ => {
                    return Err(ViceError::at(
                        ViceErrorType::Syntax,
                        "Unexpected character",
                        None,
                        self.line,
                        self.column,
                        1,
                    ));
                }
            }
        }

        tokens.push(Token::at(
            TokenKind::EOF,
            "EOF".into(),
            self.line,
            self.column,
            1,
        ));
        Ok(tokens)
    }

    fn simple(&mut self, kind: TokenKind, lexeme: &str) -> Token {
        let line = self.line;
        let column = self.column;
        self.advance();
        Token::at(kind, lexeme.into(), line, column, lexeme.chars().count())
    }

    fn string(&mut self) -> Result<Token, ViceError> {
        let line = self.line;
        let column = self.column;
        self.advance();

        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.peek() {
                    match escaped {
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        '\\' => value.push('\\'),
                        '"' => value.push('"'),
                        _ => value.push(escaped), // fallback
                    }
                    self.advance();
                }
                continue;
            }

            if ch == '"' {
                self.advance();
                let length = value.chars().count() + 2;
                return Ok(Token::at(TokenKind::String, value, line, column, length));
            }

            value.push(ch);
            self.advance();
        }

        Err(ViceError::at(
            ViceErrorType::Syntax,
            "Unterminated string",
            None,
            line,
            column,
            1,
        ))
    }

    fn number(&mut self) -> Token {
        let line = self.line;
        let column = self.column;
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if !ch.is_ascii_digit() {
                break;
            }

            value.push(ch);
            self.advance();
        }

        // Obsługa części ułamkowej
        if self.peek() == Some('.') {
            // Sprawdzamy, czy następny znak to cyfra (żeby nie pomylić z operatorem zakresu '..')
            if let Some(next) = self.source.get(self.position + 1) {
                if next.is_ascii_digit() {
                    value.push('.');
                    self.advance(); // konsumujemy '.'

                    while let Some(ch) = self.peek() {
                        if !ch.is_ascii_digit() {
                            break;
                        }
                        value.push(ch);
                        self.advance();
                    }
                }
            }
        }

        let length = value.chars().count();
        Token::at(TokenKind::Number, value, line, column, length)
    }

    fn identifier(&mut self) -> Token {
        let line = self.line;
        let column = self.column;
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if !Self::is_ident(ch) {
                break;
            }

            value.push(ch);
            self.advance();
        }

        let kind = match value.as_str() {
            "fn" => TokenKind::Func,
            "while" => TokenKind::While,
            "if" => TokenKind::If,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "else" => TokenKind::Else,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "output" => TokenKind::Output,
            "emit" => TokenKind::Emit,
            "fail" => TokenKind::Fail,
            "use" => TokenKind::Use,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Bang,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "input" => TokenKind::Input,
            "data" => TokenKind::Data,
            "return" => TokenKind::Return,

            "match" => TokenKind::Match,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "impl" => TokenKind::Impl,
            "enum" => TokenKind::Enum,
            "trait" => TokenKind::Trait,
            _ => TokenKind::Identifier,
        };

        let length = value.chars().count();
        Token::at(kind, value, line, column, length)
    }

    fn attribute(&mut self) -> Token {
        let line = self.line;
        let column = self.column;
        self.advance();

        let mut value = String::new();

        while let Some(ch) = self.peek() {
            if !Self::is_ident(ch) {
                break;
            }

            value.push(ch);
            self.advance();
        }

        let length = value.chars().count() + 1;
        Token::at(TokenKind::Attribute, value, line, column, length)
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }

            self.advance();
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.peek() != Some(expected) {
            return false;
        }

        self.advance();
        true
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn advance(&mut self) {
        if self.peek() == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self.position += 1;
    }

    fn is_ident_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    fn is_ident(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
    }
}

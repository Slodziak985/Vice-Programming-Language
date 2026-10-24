#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Func,
    Let,
    Mut,
    Output,
    Emit,
    Fail,
    Use,
    Attribute,
    Identifier,
    Number,
    String,
    Equal,
    OutputArrow,
    TypeArrow,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    EOF,
    Plus,
    Minus,
    Star,
    Slash,
    True,
    False,
    EqualEqual,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    If,
    Match,
    Colon,
    DoubleColon,
    AssignArrow,
    While,
    Input,
    And,
    Or,
    Else,
    For,
    In,
    LBracket,
    RBracket,
    Comma,
    PlusEqual,
    MinusEqual,
    Range,
    Data,
    Return,

    Bang,
    Dot,
    Break,
    Continue,
    Pipe,
    Percent,
    Impl,
    Enum,
    Trait,
    Question,
    Ampersand,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize) -> Self {
        let length = lexeme.chars().count().max(1);
        Self {
            kind,
            lexeme,
            line,
            column: 1,
            length,
        }
    }

    pub fn at(kind: TokenKind, lexeme: String, line: usize, column: usize, length: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
            length: length.max(1),
        }
    }
}

//! 词法分析

use std::{iter::Peekable, str::CharIndices};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Ident,    // 标识符
    Lambda,   // λ
    Dot,      // .
    LeftPar,  // (
    RightPar, // )
    Assign,   // =
    At,       // @
    Newline,  // \n
    Eof,
    Error,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Ident => write!(f, "identifier"),
            TokenType::Lambda => write!(f, "'λ'"),
            TokenType::Dot => write!(f, "'.'"),
            TokenType::LeftPar => write!(f, "'('"),
            TokenType::RightPar => write!(f, "')'"),
            TokenType::Assign => write!(f, "'='"),
            TokenType::At => write!(f, "'@'"),
            TokenType::Newline => write!(f, "new line"),
            TokenType::Eof => write!(f, "end of file"),
            TokenType::Error => write!(f, "unresolved symbol"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub class: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(class: TokenType, literal: impl Into<String>) -> Self {
        Self {
            class,
            literal: literal.into(),
        }
    }
}

pub trait TokenStream<'src> {
    /// 获取下一个 Token
    fn next_token(&mut self) -> Token;

    /// 是否经过换行
    fn crossed_newline(&self) -> bool;
}

#[derive(Clone)]
pub struct Lexer<'src> {
    /// 解析的文本
    literal: &'src str,
    /// 字符迭代器
    chars: Peekable<CharIndices<'src>>,
    /// 当前位置
    pos: usize,
    /// 是否经过换行，因为是换行敏感的
    crossed_newline: bool,
    /// 是否跳过了单个 '/' 防止使用 O(n) 的位置设置
    skipped_slash: bool,
}

fn is_legal_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl<'src> Lexer<'src> {
    pub fn new(literal: &'src str) -> Self {
        Self {
            literal,
            chars: literal.char_indices().peekable(),
            pos: 0,
            crossed_newline: false,
            skipped_slash: false,
        }
    }

    /// 预览字符
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    /// 消费下一个字符
    fn advance(&mut self) -> Option<char> {
        self.chars.next().and_then(|(pos, c)| {
            self.pos = pos + c.len_utf8();
            Some(c)
        })
    }

    /// 跳过空白和注释
    fn skip_whitespace(&mut self) {
        self.crossed_newline = false;
        self.skipped_slash = false;
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.advance();
                }
                Some('\n') => {
                    self.advance();
                    self.crossed_newline = true;
                }
                Some('/') => {
                    self.advance();
                    if self.peek() == Some('/') {
                        while self.peek().is_some_and(|c| c != '\n') {
                            self.advance();
                        }
                    } else {
                        self.skipped_slash = true;
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn read_ident(&mut self, start: usize) -> Token {
        while self.peek().is_some_and(|c| is_legal_ident_char(c)) {
            self.advance();
        }

        let literal = &self.literal[start..self.pos];

        Token::new(TokenType::Ident, literal)
    }
}

impl<'src> TokenStream<'src> for Lexer<'src> {
    fn next_token(&mut self) -> Token {
        self.skip_whitespace(); // 跳过空白和注释

        if self.crossed_newline {
            return Token::new(TokenType::Newline, "\n");
        }

        if self.skipped_slash {
            return Token::new(TokenType::Error, "/");
        }

        let start = self.pos;

        let Some(c) = self.advance() else {
            return Token::new(TokenType::Eof, "");
        };

        match c {
            // 单字符
            'λ' => Token::new(TokenType::Lambda, "λ"),
            '.' => Token::new(TokenType::Dot, "."),
            '(' => Token::new(TokenType::LeftPar, "("),
            ')' => Token::new(TokenType::RightPar, ")"),
            '=' => Token::new(TokenType::Assign, "="),
            '@' => Token::new(TokenType::At, "@"),

            // 标识符
            _ if is_legal_ident_char(c) => self.read_ident(start),

            // 无法识别
            _ => Token::new(TokenType::Error, &c.to_string()),
        }
    }

    fn crossed_newline(&self) -> bool {
        self.crossed_newline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(source: &str) -> Vec<TokenType> {
        let mut lexer = Lexer::new(source);
        std::iter::repeat_with(|| lexer.next_token())
            .take_while(|t| t.class != TokenType::Eof)
            .map(|token| token.class)
            .collect()
    }

    #[test]
    fn test_lexer() {
        assert_eq!(
            tokens("a = (b)"),
            vec![
                TokenType::Ident,
                TokenType::Assign,
                TokenType::LeftPar,
                TokenType::Ident,
                TokenType::RightPar
            ]
        );

        assert_eq!(
            tokens("λx. y"),
            vec![
                TokenType::Lambda,
                TokenType::Ident,
                TokenType::Dot,
                TokenType::Ident,
            ]
        );

        assert_eq!(
            tokens("@eval expr_called_猫猫"),
            vec![TokenType::At, TokenType::Ident, TokenType::Ident]
        )
    }

    #[test]
    fn test_lines() {
        assert_eq!(
            tokens("a=b // comment = @@@t t t\nccc\t=\tu"),
            vec![
                TokenType::Ident,
                TokenType::Assign,
                TokenType::Ident,
                TokenType::Newline,
                TokenType::Ident,
                TokenType::Assign,
                TokenType::Ident
            ]
        );

        assert_eq!(
            tokens("a % b"),
            vec![TokenType::Ident, TokenType::Error, TokenType::Ident,]
        );
    }
}

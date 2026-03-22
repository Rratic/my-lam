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
    Comment,  // #
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
            TokenType::Comment => write!(f, "comment"),
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
}

#[derive(Clone)]
pub struct Lexer<'src> {
    /// 解析的文本
    literal: &'src str,
    /// 字符迭代器
    chars: Peekable<CharIndices<'src>>,
    /// 当前位置
    pos: usize,
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
        loop {
            match self.peek() {
                Some(' ' | '\t' | '\r') => {
                    self.advance();
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
        self.skip_whitespace(); // 跳过空白

        let start = self.pos;

        let Some(c) = self.advance() else {
            return Token::new(TokenType::Eof, "");
        };

        match c {
            // 单字符
            '\n' => Token::new(TokenType::Newline, "\n"),
            'λ' => Token::new(TokenType::Lambda, "λ"),
            '.' => Token::new(TokenType::Dot, "."),
            '(' => Token::new(TokenType::LeftPar, "("),
            ')' => Token::new(TokenType::RightPar, ")"),
            '=' => Token::new(TokenType::Assign, "="),
            '@' => Token::new(TokenType::At, "@"),
            '#' => {
                while self.peek().is_some_and(|c| c != '\n') {
                    self.advance();
                }

                let literal = &self.literal[start..self.pos];

                Token::new(TokenType::Comment, literal)
            }

            // 标识符
            _ if is_legal_ident_char(c) => self.read_ident(start),

            // 无法识别
            _ => Token::new(TokenType::Error, &c.to_string()),
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.class == TokenType::Eof {
            None
        } else {
            Some(token)
        }
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

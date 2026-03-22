//! 语法解析
//!
//! ```bnf
//! program       ::= decl*
//!
//! decl          ::= definition
//!                 | command
//!
//! definition    ::= IDENT '=' expr
//!
//! command       ::= '@' IDENT expr
//!
//! expr          ::= IDENT | 'λ' IDENT '.' expr | expr expr
//! ```

use crate::lexer::*;
use crate::syntax::*;

pub type ParseError = String;

fn parse_error(message: impl Into<String>) -> ParseError {
    message.into()
}

pub struct Parser<'src> {
    stream: Lexer<'src>,
    current: Token,
    previous: Token,
    /// 当前 Token 与上一个 Token 之间是否有换行
    at_newline: bool,
}

impl<'src> Parser<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut stream = Lexer::new(source);
        let current = stream.next_token();
        Self {
            stream,
            current,
            previous: Token::new(TokenType::Eof, ""),
            at_newline: false,
        }
    }

    fn check(&self, class: TokenType) -> bool {
        self.current.class == class
    }

    fn advance(&mut self) -> Token {
        self.previous = std::mem::replace(&mut self.current, self.stream.next_token());
        self.at_newline = self.stream.crossed_newline();
        self.previous.clone()
    }

    fn expect(&mut self, class: TokenType) -> Result<Token, ParseError> {
        if self.check(class) {
            Ok(self.advance())
        } else {
            Err(parse_error(format!(
                "Expected {}, found {}",
                class, self.current.class
            )))
        }
    }

    fn try_match(&mut self, class: TokenType) -> bool {
        if self.check(class) {
            self.advance();
            true
        } else {
            false
        }
    }

    // ============ 解析辅助 ============

    fn parse_name(&mut self) -> Result<String, ParseError> {
        let token = self.expect(TokenType::Ident)?;
        Ok(token.literal)
    }

    // ============ 表达式解析 ============

    fn parse_atom(&mut self) -> Result<Term, ParseError> {
        if self.check(TokenType::Ident) {
            let name = self.parse_name()?;
            Ok(Term::Global(name))
        } else if self.check(TokenType::LeftPar) {
            self.advance(); // 消耗 '('
            let expr = self.parse_expr()?;
            self.expect(TokenType::RightPar)?;
            Ok(expr)
        } else {
            Err(parse_error(format!(
                "Unexpected token {:?}",
                self.current.class
            )))
        }
    }

    fn parse_app_expr(&mut self) -> Result<Term, ParseError> {
        let mut expr = self.parse_atom()?;
        loop {
            if self.check(TokenType::Ident) || self.check(TokenType::LeftPar) {
                let arg = self.parse_atom()?;
                expr = Term::App(Box::new(expr), Box::new(arg));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    pub fn parse_expr(&mut self) -> Result<Term, ParseError> {
        if self.try_match(TokenType::Lambda) {
            // λx. e
            let name = self.parse_name()?;
            self.expect(TokenType::Dot)?;
            let body = self.parse_expr()?;

            Ok(Term::Func(name, Box::new(body)))
        } else {
            self.parse_app_expr()
        }
    }

    // ============ 程序解析 ============

    fn parse_decl(&mut self) -> Result<Decl, ParseError> {
        if self.try_match(TokenType::At) {
            let operation = self.parse_name()?;
            let expr = self.parse_expr()?;
            Ok(Decl::Command(operation, expr))
        } else {
            let binded = self.parse_name()?;
            self.expect(TokenType::Assign)?;
            let expr = self.parse_expr()?;
            Ok(Decl::Definition(binded, expr))
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Decl>, ParseError> {
        let mut decls = Vec::<Decl>::new();

        while self.current.class != TokenType::Eof {
            decls.push(self.parse_decl()?);
        }

        Ok(decls)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed_expr(str: &str) -> Term {
        let mut parser = Parser::new(str);
        parser.parse_expr().unwrap()
    }

    #[test]
    fn test_parse_expr() {
        assert_eq!(parsed_expr("a"), Term::Global("a".into()));

        assert_eq!(
            parsed_expr("λx. x"),
            Term::Func("x".into(), Box::new(Term::Global("x".into())))
        );

        assert_eq!(
            parsed_expr("λx. (λy. y) λz. z"),
            Term::Func(
                "x".into(),
                Box::new(Term::App(
                    Box::new(Term::Func("y".into(), Box::new(Term::Global("y".into()))),),
                    Box::new(Term::Func("z".into(), Box::new(Term::Global("z".into()))))
                ))
            )
        );
    }

    fn parsed(str: &str) -> Vec<Decl> {
        let mut parser = Parser::new(str);
        parser.parse_program().unwrap()
    }

    #[test]
    fn test_basics() {
        assert_eq!(
            parsed("a = b"),
            vec![Decl::Definition("a".into(), Term::Global("b".into()))]
        );
    }
}

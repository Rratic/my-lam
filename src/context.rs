use crate::{elaborator::*, interpreter::*, lexer::*, parser::*, syntax::*};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    decls: HashMap<String, Term>,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            decls: HashMap::new(),
        }
    }

    fn process_inner(&mut self, decl: Decl) -> Result<String, ParseError> {
        match decl {
            Decl::Definition(name, expr) => {
                if self.decls.contains_key(&name) {
                    return Err(parse_error(format!("Redefinition of '{}'", name)));
                }

                let elaborated = elaborate(expr);
                self.decls.insert(name, elaborated);
                Ok("".into())
            }
            Decl::Command(operation, expr) => match operation.as_str() {
                "eval" => {
                    let elaborated = elaborate(expr);
                    let out = reduce_normal_order(elaborated, &self.decls)?;
                    Ok(format!("{}", out))
                }
                "simp" => {
                    let elaborated = elaborate(expr);
                    let out = simplify(elaborated, &self.decls)?;
                    Ok(format!("{}", out))
                }

                otherwise => Err(parse_error(format!("Unknown command: {}", otherwise))),
            },
        }
    }

    pub fn preprocess(&mut self, str: String) -> Vec<JsValue> {
        let lexer = Lexer::new(str.as_str());
        let tokens: Vec<Token> = lexer.collect();
        tokens
            .iter()
            .map(|t| JsValue::from_str(&t.literal))
            .collect()
    }

    pub fn process(&mut self, str: String) -> Result<String, ParseError> {
        let mut parser = Parser::new(str.as_str());
        let decls = parser.parse_program()?;
        let mut info = "".into();
        for decl in decls {
            let decl_info = self.process_inner(decl)?;
            info = format!("{}\n{}", info, decl_info);
        }
        Ok(info.trim().into())
    }
}

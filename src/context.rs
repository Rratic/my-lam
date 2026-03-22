use crate::{parser::*, syntax::*};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    decls: HashMap<String, Term>,
    current: String,
}

#[wasm_bindgen]
impl Context {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            decls: HashMap::new(),
            current: "".into(),
        }
    }

    fn process(&mut self, decl: Decl) -> Result<String, ParseError> {
        match decl {
            Decl::Definition(name, expr) => {
                let hint = if self.decls.contains_key(&name) {
                    "Overwriten definition"
                } else {
                    "Defined"
                };
                let display = format!("{}", expr);
                self.decls.insert(name, expr);
                Ok(format!("{}:\n{}", hint, display))
            }
            Decl::Command(operation, expr) => {
                Ok("wait".into())
            }
        }
    }

    pub fn process_raw(&mut self, str: String) -> Result<String, ParseError> {
        let mut parser = Parser::new(str.as_str());
        let decls = parser.parse_program()?;
        let mut info = "".into();
        for decl in decls {
            let decl_info = self.process(decl)?;
            info = format!("{}{}", info, decl_info);
        }
        Ok(info)
    }
}

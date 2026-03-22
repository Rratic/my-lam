//! 解释器

use crate::{parser::*, syntax::*};
use std::collections::{HashMap, VecDeque};

fn read_context(expr: Term, context: &HashMap<String, Term>) -> Term {
    match expr {
        Term::Global(name) => {
            if context.contains_key(&name.clone()) {
                context.get(&name).unwrap().clone()
            } else {
                Term::Global(name)
            }
        }
        Term::App(func, arg) => Term::App(Box::new(read_context(*func, context)), arg),
        _ => expr,
    }
}

fn evaluate_inner(expr: Term, env: VecDeque<Term>) -> (Term, VecDeque<Term>) {
    match expr {
        Term::Var(i) => (env[i].clone(), env),
        Term::Global(_) => (expr, env),
        Term::Func(_, body) => (*body, env),
        Term::App(func, arg) => {
            let (major, env2) = evaluate_inner(*func, env);
            let (subst, mut env3) = evaluate_inner(*arg, env2);
            env3.push_front(subst);
            evaluate_inner(major, env3)
        }
    }
}

pub fn evaluate(expr: Term, context: &HashMap<String, Term>) -> Result<Term, ParseError> {
    let read = read_context(expr, context);
    let (result, _) = evaluate_inner(read, VecDeque::new());
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let context = HashMap::<String, Term>::new();

        let expr = Term::App(
            Box::new(Term::Func("x".into(), Box::new(Term::Global("U".into())))),
            Box::new(Term::Global("M".into())),
        );

        assert_eq!(evaluate(expr, &context), Ok(Term::Global("U".into())));
    }
}

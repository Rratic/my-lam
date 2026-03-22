//! 解释器

use crate::syntax::*;
use std::collections::HashMap;

fn restore_globals(term: Term, context: &HashMap<String, Term>) -> Term {
    match term {
        Term::Var(_) => term,
        Term::Global(name) => {
            if let Some(restored) = context.get(&name) {
                restore_globals(restored.clone(), context)
            } else {
                Term::Global(name)
            }
        }
        Term::Func(name, body) => Term::func(name, restore_globals(*body, context)),
        Term::App(func, arg) => Term::app(
            restore_globals(*func, context),
            restore_globals(*arg, context),
        ),
    }
}

fn evaluate_inner(term: Term) -> Result<Term, String> {
    match term {
        Term::Var(i) => Err(format!("Unbound index: #{}", i)),
        Term::Global(_) => Ok(term), // 当作自由变量
        Term::Func(..) => Ok(term),  // 是值
        Term::App(f, a) => {
            // 先求值函数和参数
            let f_val = evaluate_inner(*f)?;
            let a_val = evaluate_inner(*a)?;
            match f_val {
                Term::Func(_, body) => {
                    // 将参数替换到函数体，然后求值
                    let substituted = body.subst(0, &a_val);
                    evaluate_inner(substituted)
                }
                _ => Err(format!("Cannot apply unknown value: {}", f_val)),
            }
        }
    }
}

pub fn evaluate(term: Term, context: &HashMap<String, Term>) -> Result<Term, String> {
    let term = restore_globals(term, context);
    evaluate_inner(term)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let mut context = HashMap::<String, Term>::new();

        assert_eq!(
            evaluate(
                Term::App(
                    Box::new(Term::Func("x".into(), Box::new(Term::Global("U".into())))),
                    Box::new(Term::Global("M".into())),
                ),
                &context
            ),
            Ok(Term::Global("U".into()))
        );

        assert_eq!(
            evaluate(
                Term::App(
                    Box::new(Term::Func("x".into(), Box::new(Term::Var(0)))),
                    Box::new(Term::Global("M".into())),
                ),
                &context
            ),
            Ok(Term::Global("M".into()))
        );

        context.insert(
            "S".into(),
            Term::func(
                "x",
                Term::func(
                    "y",
                    Term::func(
                        "z",
                        Term::app(
                            Term::app(Term::Var(2), Term::Var(0)),
                            Term::app(Term::Var(1), Term::Var(0)),
                        ),
                    ),
                ),
            ),
        );

        context.insert("K".into(), Term::func("x", Term::func("y", Term::Var(1))));

        assert_eq!(
            evaluate(
                Term::app(
                    Term::app(
                        Term::app(Term::global("S"), Term::global("K")),
                        Term::global("K"),
                    ),
                    Term::global("M")
                ),
                &context
            ),
            Ok(Term::Global("M".into()))
        );
    }
}

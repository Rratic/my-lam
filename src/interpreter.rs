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

fn evaluate_inner(term: Term) -> Term {
    match term {
        Term::Var(_) => term,
        Term::Global(_) => term, // 当作自由变量
        Term::Func(name, body) => {
            let body_val = evaluate_inner(*body);
            match body_val {
                Term::App(f, a) => {
                    // eta-reduce
                    if Term::Var(0) == *a && f.irrelevant(0) {
                        let inner = *f;
                        evaluate_inner(inner)
                    } else {
                        Term::func(name, Term::App(f, a))
                    }
                }
                _ => Term::func(name, body_val),
            }
        }
        Term::App(f, a) => {
            // 先求值函数和参数
            let f_val = evaluate_inner(*f);
            match f_val {
                Term::Func(_, body) => {
                    // 将参数替换到函数体，然后求值
                    let substituted = body.subst(0, a.as_ref());
                    evaluate_inner(substituted)
                }
                _ => Term::app(f_val, *a),
            }
        }
    }
}

pub fn evaluate(term: Term, context: &HashMap<String, Term>) -> Result<Term, String> {
    let term = restore_globals(term, context);
    Ok(evaluate_inner(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate() {
        let context = HashMap::<String, Term>::new();

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
    }

    #[test]
    fn test_eta_reduce() {
        let context = HashMap::<String, Term>::new();

        assert_eq!(
            evaluate(
                Term::func("x", Term::app(Term::global("f"), Term::Var(0))),
                &context
            ),
            Ok(Term::global("f"))
        );
    }

    #[test]
    fn test_complicated() {
        let mut context = HashMap::<String, Term>::new();

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
                    Term::app(Term::global("S"), Term::global("K")),
                    Term::global("K"),
                ),
                &context
            ),
            Ok(Term::func("z", Term::Var(0)))
        );
    }
}

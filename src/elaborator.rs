//! 繁饰器

use crate::syntax::*;

fn elaborate_with(term: &Term, stack: &mut Vec<String>) -> Term {
    match term {
        Term::Var(_) => term.clone(),
        Term::Global(name) => {
            let len = stack.len();
            for i in 0..len {
                if stack[i] == *name {
                    return Term::Var((len - i - 1) as i32);
                }
            }
            term.clone()
        }
        Term::Func(name, body) => {
            stack.push(name.clone());
            let elaborated_body = elaborate_with(body.as_ref(), stack);
            Term::Func(stack.pop().unwrap(), Box::new(elaborated_body))
        }
        Term::App(func, arg) => {
            let elaborated_func = elaborate_with(func.as_ref(), stack);
            let elaborated_arg = elaborate_with(arg.as_ref(), stack);
            Term::App(Box::new(elaborated_func), Box::new(elaborated_arg))
        }
    }
}

pub fn elaborate(term: Term) -> Term {
    elaborate_with(&term, &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elaborate() {
        let ex1 = Term::Func("x".into(), Box::new(Term::Global("E".into())));
        assert_eq!(elaborate(ex1.clone()), ex1);

        assert_eq!(
            elaborate(Term::Func("x".into(), Box::new(Term::Global("x".into())))),
            Term::Func("x".into(), Box::new(Term::Var(0)))
        );

        assert_eq!(
            elaborate(Term::Func(
                "x".into(),
                Box::new(Term::App(
                    Box::new(Term::Func("y".into(), Box::new(Term::Global("y".into()))),),
                    Box::new(Term::Func("z".into(), Box::new(Term::Global("x".into()))))
                ))
            )),
            Term::Func(
                "x".into(),
                Box::new(Term::App(
                    Box::new(Term::Func("y".into(), Box::new(Term::Var(0))),),
                    Box::new(Term::Func("z".into(), Box::new(Term::Var(1))))
                ))
            )
        )
    }
}

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

/// 尝试单步左最外 β 归约，若无法归约则返回 `None`
fn try_beta_normal_order_step(term: &Term) -> Option<Term> {
    match term {
        Term::Var(_) | Term::Global(_) => None,
        Term::Func(name, body) => {
            try_beta_normal_order_step(body).map(|b| Term::func(name.clone(), b))
        }
        Term::App(f, a) => {
            if let Term::Func(_, body) = f.as_ref() {
                return Some(body.subst(0, a));
            }
            try_beta_normal_order_step(f)
                .map(|f2| Term::app(f2, *a.clone()))
                .or_else(|| try_beta_normal_order_step(a).map(|a2| Term::app(*f.clone(), a2)))
        }
    }
}

/// 反复作正规序 β 归约直到无法再归约；不保证终止
fn reduce_normal_order_inner(term: Term) -> Term {
    let mut t = term;
    loop {
        let Some(next) = try_beta_normal_order_step(&t) else {
            break;
        };
        if next == t {
            break;
        }
        t = next;
    }
    t
}

/// 尝试单步 η 归约，若无法归约则返回 `None`
fn try_eta_step(term: &Term) -> Option<Term> {
    match term {
        Term::Var(_) | Term::Global(_) => None,
        Term::Func(name, body) => {
            if let Term::App(f, a) = body.as_ref() {
                if matches!(a.as_ref(), Term::Var(0)) && f.var_not_free(0) {
                    return Some(f.shift(1, -1));
                }
            }
            try_eta_step(body).map(|b| Term::func(name.clone(), b))
        }
        Term::App(f, a) => try_eta_step(f)
            .map(|f2| Term::app(f2, *a.clone()))
            .or_else(|| try_eta_step(a).map(|a2| Term::app(*f.clone(), a2))),
    }
}

/// 反复作 η 归约直到无法再归约
fn eta_normalize_inner(term: Term) -> Term {
    let mut t = term;
    loop {
        let Some(next) = try_eta_step(&t) else {
            break;
        };
        if next == t {
            break;
        }
        t = next;
    }
    t
}

/// 反复作正规序 β 归约与 η 归约直到不动，尽量简化表达式
fn simplify_inner(term: Term) -> Term {
    let mut t = term;
    loop {
        let before = t.clone();
        t = reduce_normal_order_inner(t);
        t = eta_normalize_inner(t);
        if t == before {
            break;
        }
    }
    t
}

/// @eval 命令实现
pub fn reduce_normal_order(term: Term, context: &HashMap<String, Term>) -> Result<Term, String> {
    let term = restore_globals(term, context);
    Ok(reduce_normal_order_inner(term))
}

/// @simp 命令实现
pub fn simplify(term: Term, context: &HashMap<String, Term>) -> Result<Term, String> {
    let term = restore_globals(term, context);
    Ok(simplify_inner(term))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplify_basic_applications() {
        let context = HashMap::<String, Term>::new();

        assert_eq!(
            simplify(
                Term::App(
                    Box::new(Term::Func("x".into(), Box::new(Term::Global("U".into())))),
                    Box::new(Term::Global("M".into())),
                ),
                &context
            ),
            Ok(Term::Global("U".into()))
        );

        assert_eq!(
            simplify(
                Term::App(
                    Box::new(Term::Func("x".into(), Box::new(Term::Var(0)))),
                    Box::new(Term::Global("M".into())),
                ),
                &context
            ),
            Ok(Term::Global("M".into()))
        );
    }

    /// `reduce_normal_order` 与 `simplify` 在这些例子上结果相同
    #[test]
    fn reduce_normal_order_basic_applications() {
        let context = HashMap::<String, Term>::new();

        assert_eq!(
            reduce_normal_order(
                Term::App(
                    Box::new(Term::Func("x".into(), Box::new(Term::Global("U".into())))),
                    Box::new(Term::Global("M".into())),
                ),
                &context
            ),
            Ok(Term::Global("U".into()))
        );

        assert_eq!(
            reduce_normal_order(
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
            simplify(
                Term::func("x", Term::app(Term::global("f"), Term::Var(0))),
                &context
            ),
            Ok(Term::global("f"))
        );
    }

    /// SK 组合子环境
    fn context_s_and_k() -> HashMap<String, Term> {
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
        context
    }

    #[test]
    fn simplify_skk_is_identity() {
        let context = context_s_and_k();
        assert_eq!(
            simplify(
                Term::app(
                    Term::app(Term::global("S"), Term::global("K")),
                    Term::global("K"),
                ),
                &context
            ),
            Ok(Term::func("z", Term::Var(0)))
        );
    }

    /// `reduce_normal_order` 给出的结果相同
    #[test]
    fn reduce_normal_order_skk_is_identity() {
        let context = context_s_and_k();
        assert_eq!(
            reduce_normal_order(
                Term::app(
                    Term::app(Term::global("S"), Term::global("K")),
                    Term::global("K"),
                ),
                &context
            ),
            Ok(Term::func("z", Term::Var(0)))
        );
    }

    /// 使用正规序归约时，先归约最外层
    #[test]
    fn normal_order_outer_first() {
        let context = HashMap::new();
        // (λx.x) ((λy.y) g)  →  ((λy.y) g)  →  g
        let t = Term::app(
            Term::func("x", Term::Var(0)),
            Term::app(Term::func("y", Term::Var(0)), Term::global("g")),
        );
        assert_eq!(
            reduce_normal_order(t.clone(), &context),
            Ok(Term::global("g"))
        );
        assert_eq!(simplify(t, &context), Ok(Term::global("g")));
    }

    /// 使用正规序归约时，仅作正规序 β 归约，不作 η 归约
    #[test]
    fn beta_only_vs_simplify_eta() {
        let context = HashMap::new();
        let t = Term::func("x", Term::app(Term::global("h"), Term::Var(0)));
        let beta = reduce_normal_order(t.clone(), &context).unwrap();
        assert_eq!(beta, t);
        assert_eq!(simplify(t, &context).unwrap(), Term::global("h"));
    }

    /// 使用简化时，会作正规序 β 归约与 η 归约的交替迭代
    #[test]
    fn simplify_interleaves_beta_eta() {
        let context = HashMap::new();
        // (λx. (λy.y) x) z  —β→  (λy.y) z  —β→  z
        let t = Term::app(
            Term::func("x", Term::app(Term::func("y", Term::Var(0)), Term::Var(0))),
            Term::global("z"),
        );
        assert_eq!(simplify(t, &context).unwrap(), Term::global("z"));
    }

    #[test]
    fn reduce_normal_order_matches_simplify_without_eta_need() {
        let mut context = HashMap::new();
        context.insert("I".into(), Term::func("x", Term::Var(0)));
        let t = Term::app(Term::global("I"), Term::global("a"));
        let r = reduce_normal_order(t.clone(), &context).unwrap();
        let s = simplify(t, &context).unwrap();
        assert_eq!(r, s);
        assert_eq!(r, Term::global("a"));
    }

    fn church_arith_context() -> (HashMap<String, Term>, Term, Term) {
        fn church_1() -> Term {
            Term::func("f", Term::func("s", Term::app(Term::Var(1), Term::Var(0))))
        }
        fn church_2() -> Term {
            Term::func(
                "f",
                Term::func(
                    "s",
                    Term::app(Term::Var(1), Term::app(Term::Var(1), Term::Var(0))),
                ),
            )
        }
        fn church_3() -> Term {
            Term::func(
                "f",
                Term::func(
                    "s",
                    Term::app(
                        Term::Var(1),
                        Term::app(Term::Var(1), Term::app(Term::Var(1), Term::Var(0))),
                    ),
                ),
            )
        }
        fn church_succ() -> Term {
            Term::func(
                "n",
                Term::func(
                    "f",
                    Term::func(
                        "s",
                        Term::app(
                            Term::Var(1),
                            Term::app(Term::app(Term::Var(2), Term::Var(1)), Term::Var(0)),
                        ),
                    ),
                ),
            )
        }
        fn church_plus() -> Term {
            Term::func(
                "n",
                Term::func(
                    "m",
                    Term::app(Term::app(Term::Var(1), Term::global("Succ")), Term::Var(0)),
                ),
            )
        }

        let mut context = HashMap::new();
        let c3 = church_3();
        context.insert("1".into(), church_1());
        context.insert("2".into(), church_2());
        context.insert("3".into(), c3.clone());
        context.insert("Succ".into(), church_succ());
        context.insert("Plus".into(), church_plus());

        let t = Term::app(
            Term::app(Term::global("Plus"), Term::global("1")),
            Term::global("2"),
        );
        (context, t, c3)
    }

    /// 仅正规序 β 即可将 `Plus 1 2` 归约到 `3`
    #[test]
    fn church_plus_one_two_is_three_normal_order() {
        let (context, t, c3) = church_arith_context();
        assert_eq!(reduce_normal_order(t, &context), Ok(c3));
    }

    /// `simplify` 与正规序在此例上等价于 Church 数 `3`
    #[test]
    fn church_plus_one_two_is_three_simplify() {
        let (context, t, c3) = church_arith_context();
        assert_eq!(simplify(t, &context), Ok(c3));
    }
}

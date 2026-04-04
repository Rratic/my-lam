#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(usize),              // 使用 de Bruijn 索引
    Global(String),          // 全局变量
    Func(String, Box<Term>), // 标记参数名
    App(Box<Term>, Box<Term>),
}

impl Term {
    #[cfg(test)]
    pub fn global(name: impl Into<String>) -> Self {
        Term::Global(name.into())
    }

    pub fn func(name: impl Into<String>, body: Term) -> Self {
        Term::Func(name.into(), Box::new(body))
    }

    pub fn app(func: impl Into<Box<Term>>, arg: Term) -> Self {
        Term::App(func.into(), Box::new(arg))
    }
}

impl Term {
    /// `index` 级 de Bruijn 变量是否不在 `self` 中自由出现（进入 `Func` 时索引加一）。
    pub fn var_not_free(&self, index: usize) -> bool {
        match self {
            Term::Var(i) => *i != index,
            Term::Global(_) => true,
            Term::Func(_, body) => body.var_not_free(index + 1),
            Term::App(func, arg) => func.var_not_free(index) && arg.var_not_free(index),
        }
    }

    pub fn shift(&self, cutoff: usize, amount: isize) -> Term {
        match self {
            Term::Var(i) => {
                if *i >= cutoff {
                    Term::Var((((*i) as isize) + amount) as usize)
                } else {
                    Term::Var(*i)
                }
            }
            Term::Global(name) => Term::Global(name.clone()),
            Term::Func(name, body) => Term::func(name, body.shift(cutoff + 1, amount)),
            Term::App(func, arg) => {
                Term::app(func.shift(cutoff, amount), arg.shift(cutoff, amount))
            }
        }
    }

    pub fn subst(&self, index: usize, term: &Term) -> Term {
        match self {
            Term::Var(i) => match (*i).cmp(&index) {
                std::cmp::Ordering::Equal => term.clone(),
                std::cmp::Ordering::Greater => Term::Var(*i - 1),
                std::cmp::Ordering::Less => Term::Var(*i),
            },
            Term::Global(name) => Term::Global(name.clone()),
            Term::Func(name, body) => {
                let shifted = term.shift(0, 1);
                Term::func(name, body.subst(index + 1, &shifted))
            }
            Term::App(func, arg) => Term::app(func.subst(index, term), arg.subst(index, term)),
        }
    }
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Term::Var(i) => write!(f, "#{}", i),
            Term::Global(name) => write!(f, "{}", name),
            Term::Func(name, body) => write!(f, "(λ{}. {})", name, body),
            Term::App(func, arg) => write!(f, "({} {})", func, arg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decl {
    Definition(String, Term),
    Command(String, Term),
}

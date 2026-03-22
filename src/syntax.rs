#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    Var(usize),              // 使用 de Bruijn 索引
    Global(String),          // 全局变量
    Func(String, Box<Term>), // 标记参数名
    App(Box<Term>, Box<Term>),
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

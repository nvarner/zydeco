use super::syntax::*;

pub trait FmtWithArgs {
    fn fmt_with_args(&self, args: Args) -> String;
}

#[derive(Clone, Copy)]
pub struct Args {
    indent_unit: usize,
    pub indent: usize,
}
impl Args {
    pub fn new(indent_unit: usize) -> Self {
        Self {
            indent_unit,
            indent: 0,
        }
    }
    pub fn indent(&self) -> Self {
        Self {
            indent: self.indent + self.indent_unit,
            ..self.clone()
        }
    }
    pub fn force_space(&self) -> String {
        format!("\n{}", " ".repeat(self.indent))
    }
}

impl<Ann> FmtWithArgs for Program<Ann> {
    fn fmt_with_args(&self, args: Args) -> String {
        self.comp.fmt_with_args(args)
    }
}

impl<Ann> FmtWithArgs for Value<Ann> {
    fn fmt_with_args(&self, args: Args) -> String {
        match self {
            Value::Var(x, _) => format!("{}", x.fmt_with_args(args)),
            Value::Thunk(e, _) => format!("{{ {} }}", e.fmt_with_args(args)),
            Value::Ctor(ctor, vs, _) => format!(
                "{}({})",
                ctor.fmt_with_args(args),
                vs.into_iter()
                    .map(|v| v.fmt_with_args(args))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::Bool(b, _) => format!("{}", b),
            Value::Int(n, _) => format!("{}", n),
        }
    }
}

impl<Ann> FmtWithArgs for Compute<Ann> {
    fn fmt_with_args(&self, args: Args) -> String {
        match self {
            Compute::Let { binding, body, .. } => {
                let (x, v) = binding;
                format!(
                    "let {} = {};{}{}",
                    x.fmt_with_args(args),
                    v.fmt_with_args(args),
                    args.force_space(),
                    body.fmt_with_args(args)
                )
            }
            Compute::Do { binding, body, .. } => {
                let (x, v) = binding;
                format!(
                    "do {} <- {};{}{}",
                    x.fmt_with_args(args),
                    v.fmt_with_args(args),
                    args.force_space(),
                    body.fmt_with_args(args)
                )
            }
            Compute::Force(v, _) => {
                format!("!{}", v.fmt_with_args(args))
            }
            Compute::Return(v, _) => {
                format!("ret {}", v.fmt_with_args(args))
            }
            Compute::Lam { arg, body, .. } => {
                let (x, t) = arg;
                format!(
                    "fn ({}: {}) -> {}",
                    x.fmt_with_args(args),
                    t.fmt_with_args(args),
                    body.fmt_with_args(args)
                )
            }
            Compute::App(e, v, _) => {
                format!("{} {}", e.fmt_with_args(args), v.fmt_with_args(args),)
            }
            Compute::If { cond, thn, els, .. } => {
                format!(
                    "if {}: {} else: {}",
                    cond.fmt_with_args(args),
                    thn.fmt_with_args(args),
                    els.fmt_with_args(args)
                )
            }
            Compute::Match { scrut, cases, .. } => {
                format!("match {} {}", scrut.fmt_with_args(args), {
                    let args = args.indent();
                    cases
                        .into_iter()
                        .map(|(ctor, vs, e)| {
                            format!(
                                "{}| {}({}) -> {}",
                                args.force_space(),
                                ctor.fmt_with_args(args),
                                vs.into_iter()
                                    .map(|v| v.fmt_with_args(args))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                e.fmt_with_args(args),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                })
            }
            Compute::CoMatch { cases, .. } => {
                format!("comatch {}", {
                    let args = args.indent();
                    cases
                        .into_iter()
                        .map(|(dtor, vs, e)| {
                            format!(
                                "{}.{}({}) -> {}",
                                args.force_space(),
                                dtor.fmt_with_args(args),
                                vs.into_iter()
                                    .map(|v| v.fmt_with_args(args))
                                    .collect::<Vec<_>>()
                                    .join(", "),
                                e.fmt_with_args(args),
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                })
            }
        }
    }
}

impl<Ann> FmtWithArgs for TValue<Ann> {
    fn fmt_with_args(&self, args: Args) -> String {
        match self {
            TValue::Var(x, _) => format!("{}", x.fmt_with_args(args)),
            TValue::Comp(c, _) => format!("Comp({})", c.fmt_with_args(args)),
            TValue::Bool(_) => format!("Bool"),
            TValue::Int(_) => format!("Int"),
        }
    }
}

impl<Ann> FmtWithArgs for TCompute<Ann> {
    fn fmt_with_args(&self, args: Args) -> String {
        match self {
            TCompute::Var(x, _) => format!("{}", x.fmt_with_args(args)),
            TCompute::Ret(v, _) => format!("Ret({})", v.fmt_with_args(args)),
            TCompute::Lam(t, c, _) => {
                format!("{} -> {}", t.fmt_with_args(args), c.fmt_with_args(args))
            }
        }
    }
}

pub trait FmtDefault: FmtWithArgs {
    fn fmt(&self) -> String {
        self.fmt_with_args(Args::new(2))
    }
}

impl FmtDefault for Program<()> {}
impl FmtDefault for TValue<()> {}
impl FmtDefault for TCompute<()> {}
impl FmtDefault for Value<()> {}
impl FmtDefault for Compute<()> {}

macro_rules! var_fmt {
    ($Var:ident) => {
        impl<Ann> FmtWithArgs for $Var<Ann> {
            fn fmt_with_args(&self, _args: Args) -> String {
                format!("{}", self.name())
            }
        }
        impl<Ann> FmtDefault for $Var<Ann> {}
    };
}

var_fmt!(TVar);
var_fmt!(Ctor);
var_fmt!(Dtor);
var_fmt!(VVar);
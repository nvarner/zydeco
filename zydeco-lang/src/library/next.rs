pub mod syntax {
    use crate::dynamics::next::syntax as ds;
    pub use crate::syntax::{env::Env, *};
    use indexmap::IndexMap;
    use std::io::{BufRead, Write};
    use std::rc::Rc;
    use zydeco_derive::EnumGenerator;

    /* ---------------------------------- Term ---------------------------------- */

    #[derive(EnumGenerator, Clone)]
    pub enum TermValue {
        Var(TermV),
        Thunk(Thunk<TC>),
        Ctor(Ctor<CtorV, TV>),
        Literal(Literal),
    }
    type TV = Rc<TermValue>;
    impl ValueT for TermValue {}

    pub type PrimComp = fn(
        Vec<ds::TermValue>,
        &mut (dyn BufRead),
        &mut (dyn Write),
        &[String],
    ) -> Result<TermComputation, i32>;

    #[derive(Clone)]
    pub struct Prim {
        pub arity: u64,
        pub body: PrimComp,
    }

    #[derive(EnumGenerator, Clone)]
    pub enum TermComputation {
        Ret(Ret<TV>),
        Force(Force<TV>),
        Let(Let<TermV, TV, TC>),
        Do(Do<TermV, TC, TC>),
        Rec(Rec<TermV, TC>),
        Match(Match<CtorV, TermV, TV, TC>),
        CoMatch(CoMatch<DtorV, TermV, TC>),
        Dtor(Dtor<TC, DtorV, TV>),
        Prim(Prim),
    }
    type TC = Rc<TermComputation>;
    impl ComputationT for TermComputation {}

    #[derive(EnumGenerator, Clone)]
    pub enum Term {
        Val(TermValue),
        Comp(TermComputation),
    }

    /* --------------------------------- Module --------------------------------- */

    #[derive(Clone)]
    pub struct Module {
        pub name: Option<String>,
        pub define: IndexMap<TermV, TermValue>,
        pub entry: TermComputation,
    }
}

mod link {
    use super::syntax::*;
    use crate::rc;
    use crate::statics::next::syntax as ss;

    impl From<ss::Module> for Module {
        fn from(m: ss::Module) -> Self {
            Self {
                name: m.name,
                define: m
                    .define
                    .into_iter()
                    .map(|def| {
                        let ss::Define { name, ty: _, def } = def.inner();
                        if let Some(def) = def {
                            (name, def.inner_ref().into())
                        } else {
                            // Todo: use extern field
                            todo!()
                        }
                    })
                    .collect(),
                entry: m.entry.inner_ref().into(),
            }
        }
    }

    impl From<&ss::TermValue> for TermValue {
        fn from(v: &ss::TermValue) -> Self {
            match v {
                ss::TermValue::TermAnn(TermAnn { body, ty: _ }) => {
                    body.inner_ref().into()
                }
                ss::TermValue::Var(x) => x.clone().into(),
                ss::TermValue::Thunk(Thunk(e)) => {
                    Thunk(rc!(e.inner_ref().into())).into()
                }
                ss::TermValue::Ctor(Ctor { ctor, args }) => {
                    let args = args
                        .iter()
                        .map(|v| rc!(v.inner_ref().into()))
                        .collect();
                    Ctor { ctor: ctor.clone(), args }.into()
                }
                ss::TermValue::Literal(l) => l.clone().into(),
            }
        }
    }

    impl From<&ss::TermComputation> for TermComputation {
        fn from(e: &ss::TermComputation) -> Self {
            match e {
                ss::TermComputation::TermAnn(TermAnn { body, ty: _ }) => {
                    body.inner_ref().into()
                }
                ss::TermComputation::Ret(Ret(v)) => {
                    Ret(rc!(v.inner_ref().into())).into()
                }
                ss::TermComputation::Force(Force(v)) => {
                    Force(rc!(v.inner_ref().into())).into()
                }
                ss::TermComputation::Let(Let { var, def, body }) => {
                    let (def, body) =
                        rc!(def.inner_ref().into(), body.inner_ref().into());
                    Let { var: var.clone(), def, body }.into()
                }
                ss::TermComputation::Do(Do { var, comp, body }) => {
                    let (comp, body) =
                        rc!(comp.inner_ref().into(), body.inner_ref().into());
                    Do { var: var.clone(), comp, body }.into()
                }
                ss::TermComputation::Rec(Rec { var, body }) => {
                    let body = rc!(body.inner_ref().into());
                    Rec { var: var.clone(), body }.into()
                }
                ss::TermComputation::Match(Match { scrut, arms }) => {
                    let scrut = rc!(scrut.inner_ref().into());
                    let arms = arms
                        .iter()
                        .map(|Matcher { ctor, vars, body }| {
                            let body = rc!(body.inner_ref().into());
                            Matcher {
                                ctor: ctor.clone(),
                                vars: vars.clone(),
                                body,
                            }
                        })
                        .collect();
                    Match { scrut, arms }.into()
                }
                ss::TermComputation::CoMatch(CoMatch { arms }) => {
                    let arms = arms
                        .iter()
                        .map(|CoMatcher { dtor, vars, body }| {
                            let body = rc!(body.inner_ref().into());
                            CoMatcher {
                                dtor: dtor.clone(),
                                vars: vars.clone(),
                                body,
                            }
                        })
                        .collect();
                    CoMatch { arms }.into()
                }
                ss::TermComputation::Dtor(Dtor { body, dtor, args }) => {
                    let body = rc!(body.inner_ref().into());
                    let args = args
                        .iter()
                        .map(|arg| rc!(arg.inner_ref().into()))
                        .collect();
                    Dtor { body, dtor: dtor.clone(), args }.into()
                }
            }
        }
    }
}
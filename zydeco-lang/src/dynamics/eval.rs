use super::syntax::{Thunk as SemThunk, *};
use crate::{rc, utils::fmt::FmtArgs};
use im::Vector;
use std::{
    io::{BufRead, Write},
    rc::Rc,
};

pub trait Eval<'rt>: Sized + FmtArgs {
    type Out;
    fn step<'e>(self, runtime: &'e mut Runtime<'rt>) -> Step<Self, Self::Out>;
    fn eval<'e>(self, runtime: &'e mut Runtime<'rt>) -> Self::Out {
        let mut res = self;
        loop {
            match res.step(runtime) {
                Step::Done(out) => break out,
                Step::Step(next) => res = next,
            }
        }
    }
}

pub enum Step<T, Out> {
    Done(Out),
    Step(T),
}

impl<'rt> Runtime<'rt> {
    pub fn new(
        input: &'rt mut dyn BufRead, output: &'rt mut dyn Write,
        args: &'rt [String],
    ) -> Self {
        Runtime { input, output, args, stack: Vector::new(), env: Env::new() }
    }
}

impl<'rt> Eval<'rt> for ls::ZVal {
    type Out = SemVal;

    fn step<'e>(self, runtime: &'e mut Runtime<'rt>) -> Step<Self, Self::Out> {
        match self {
            ls::ZVal::Var(var) => Step::Done({
                runtime
                    .env
                    .lookup(&var)
                    .expect("variable does not exist")
                    .clone()
            }),
            ls::ZVal::Thunk(ls::Thunk(body)) => Step::Done(
                super::syntax::Thunk { body, env: runtime.env.clone() }.into(),
            ),
            ls::ZVal::Ctor(ls::Ctor { ctor, args }) => {
                let args = args
                    .iter()
                    .map(|arg| rc!(arg.as_ref().clone().eval(runtime)))
                    .collect();
                Step::Done(ls::Ctor { ctor, args }.into())
            }
            ls::ZVal::Literal(lit) => Step::Done(lit.into()),
            ls::ZVal::SemValue(sem) => Step::Done(sem),
        }
    }
}

impl<'rt> Eval<'rt> for ls::ZComp {
    type Out = ProgKont;

    fn step<'e>(self, runtime: &'e mut Runtime<'rt>) -> Step<Self, Self::Out> {
        match self {
            ls::ZComp::Ret(ls::Ret(v)) => {
                let v = v.as_ref().clone().eval(runtime);
                match runtime.stack.pop_back() {
                    Some(SemComp::Kont(comp, env, var)) => {
                        let env = env.update(var, v);
                        runtime.env = env;
                        Step::Step(comp.as_ref().clone())
                    }
                    None => Step::Done(ProgKont::Ret(v)),
                    _ => panic!("Kont not at stacktop"),
                }
            }
            ls::ZComp::Force(ls::Force(v)) => {
                let v = v.as_ref().clone().eval(runtime);
                let SemVal::Thunk(thunk) = v else {
                    panic!("Force on non-thunk")
                };
                runtime.env = thunk.env;
                Step::Step(thunk.body.as_ref().clone())
            }
            ls::ZComp::Let(ls::Let { var, def, body }) => {
                let def = def.as_ref().clone().eval(runtime);
                let env = runtime.env.update(var, def);
                runtime.env = env;
                Step::Step(body.as_ref().clone())
            }
            ls::ZComp::Do(ls::Do { var, comp, body }) => {
                runtime.stack.push_back(SemComp::Kont(
                    body,
                    runtime.env.clone(),
                    var,
                ));
                Step::Step(comp.as_ref().clone())
            }
            ls::ZComp::Rec(e) => {
                let ls::Rec { var, body } = e.clone();
                let env = runtime.env.update(
                    var,
                    SemThunk { body: rc!(e.into()), env: runtime.env.clone() }
                        .into(),
                );
                runtime.env = env;
                Step::Step(body.as_ref().clone())
            }
            ls::ZComp::Match(ls::Match { scrut, arms }) => {
                let scrut = scrut.as_ref().clone().eval(runtime);
                let SemVal::Ctor(ls::Ctor { ctor, args }) = scrut else {
                    panic!("Match on non-ctor")
                };
                let ls::Matcher { ctor: _, vars, body } = arms
                    .into_iter()
                    .find(|arm| arm.ctor == ctor)
                    .expect("no matching arm");
                for (var, arg) in vars.into_iter().zip(args.into_iter()) {
                    let env = runtime.env.update(var, arg.as_ref().clone());
                    runtime.env = env;
                }
                Step::Step(body.as_ref().clone())
            }
            ls::ZComp::CoMatch(ls::CoMatch { arms }) => {
                let Some(SemComp::Dtor(dtor, args)) = runtime.stack.pop_back() else {
                    panic!("CoMatch on non-Dtor")
                };
                let ls::CoMatcher { dtor: _, vars, body } = arms
                    .into_iter()
                    .find(|arm| arm.dtor == dtor)
                    .expect("no matching arm");
                for (var, arg) in vars.into_iter().zip(args.into_iter()) {
                    let env = runtime.env.update(var, arg.as_ref().clone());
                    runtime.env = env;
                }
                Step::Step(body.as_ref().clone())
            }
            ls::ZComp::Dtor(ls::Dtor { body, dtor, args }) => {
                let args = args
                    .iter()
                    .map(|arg| Rc::new(arg.as_ref().clone().eval(runtime)))
                    .collect();
                runtime.stack.push_back(SemComp::Dtor(dtor, args));
                Step::Step(body.as_ref().clone())
            }
            ls::ZComp::Prim(ls::Prim { arity, body }) => {
                let mut args = Vec::new();
                for _ in 0..arity {
                    let Some(SemComp::Dtor(_, arg)) = runtime.stack.pop_back() else {
                        panic!("Prim on non-Dtor")
                    };
                    args.push(arg.first().expect("empty arg").as_ref().clone());
                }
                match body(args, runtime.input, runtime.output, runtime.args) {
                    Ok(e) => Step::Step(e),
                    Err(exit_code) => Step::Done(ProgKont::ExitCode(exit_code)),
                }
            }
        }
    }
}

impl<'rt> Eval<'rt> for ls::Module {
    type Out = Module;

    fn step<'e>(self, runtime: &'e mut Runtime<'rt>) -> Step<Self, Self::Out> {
        for (x, v) in self.define {
            let v = v.clone().eval(runtime);
            let env = runtime.env.update(x, v);
            runtime.env = env;
        }
        Step::Done(Module { name: self.name })
    }
}

impl<'rt> Eval<'rt> for ls::Program {
    type Out = Program;

    fn step<'e>(self, runtime: &'e mut Runtime<'rt>) -> Step<Self, Self::Out> {
        let module = self.module.eval(runtime);
        let prog_kont = self.entry.eval(runtime);
        Step::Done(Program { module, entry: prog_kont })
    }
}

impl Program {
    pub fn run<'rt>(p: ls::Program, runtime: &mut Runtime<'rt>) -> Self {
        p.eval(runtime)
    }
}

// use lalrpop_util::lalrpop_mod;
// lalrpop_mod!(pub parser, "/parse/next/parser.rs");
pub use crate::{syntax::Ann, syntax::*};
use zydeco_derive::EnumGenerator;

/* ---------------------------------- Kind ---------------------------------- */

pub use crate::syntax::Kind;

/* ---------------------------------- Type ---------------------------------- */


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeApp(pub Box<Type>, pub Box<Type>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeArrow(pub Box<Type>, pub Box<Type>);

#[derive(EnumGenerator, Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Basic(TCtor),
    App(TypeApp),
    Arrow(TypeArrow),
}
impl TypeT for Type {}

/* ---------------------------------- Term ---------------------------------- */

#[derive(EnumGenerator, Clone, Debug, PartialEq, Eq)]
pub enum TermValue {
    TermAnn(TermAnn<TValue, Ann<Type>>),
    Var(TermV),
    Thunk(Thunk<TComp>),
    Ctor(Ctor<CtorV, Ann<TermValue>>),
    Literal(Literal),
}
type TValue = Box<Ann<TermValue>>;
impl ValueT for TermValue {}

type TermPattern = (TermV, Option<Ann<Type>>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Abstraction {
    pub params: Vec<TermPattern>,
    pub body: TComp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub expr_in: TComp,
    pub args: Vec<Ann<TermValue>>,
}

#[derive(EnumGenerator, Clone, Debug, PartialEq, Eq)]
pub enum TermComputation {
    TermAnn(TermAnn<TComp, Ann<Type>>),
    Ret(Ret<TValue>),
    Force(Force<TValue>),
    Let(Let<TermPattern, TValue, TComp>),
    Do(Do<TermPattern, TComp, TComp>),
    Rec(Rec<TermPattern, TComp>),
    Match(Match<CtorV, TermV, TValue, Ann<TermComputation>>),
    Function(Abstraction),
    Application(Application),
    CoMatch(CoMatch<DtorV, TermV, Ann<TermComputation>>),
    Dtor(Dtor<TComp, DtorV, Ann<TermValue>>),
}
type TComp = Box<Ann<TermComputation>>;
impl ComputationT for TermComputation {}

#[derive(EnumGenerator, Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Val(TermValue),
    Comp(TermComputation),
}

/* --------------------------------- Module --------------------------------- */

#[derive(EnumGenerator, Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    Data(Data<TypeV, CtorV, Ann<Type>>),
    Codata(Codata<TypeV, DtorV, Ann<Type>>),
    Define(Define<TermV, Option<Ann<Type>>, TValue>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub name: Option<String>,
    pub declarations: Vec<DeclSymbol<Declaration>>,
    pub entry: Ann<TermComputation>,
}
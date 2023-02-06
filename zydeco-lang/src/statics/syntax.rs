use crate::{syntax::*, utils::ann::Ann};
use enum_dispatch::enum_dispatch;
use std::{collections::HashMap, rc::Rc};

/* ---------------------------------- Kind ---------------------------------- */

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    VType,
    CType,
}
impl KindT for Kind {}

/* ---------------------------------- Type ---------------------------------- */

#[enum_dispatch(TypeT)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    TypeAnn(Ann<TypeAnn<T, Ann<Kind>>>),
    TCtor(Ann<TypeApp<TCtor, T>>),
}
type T = Rc<Type>;
impl TypeT for Type {}

/* ---------------------------------- Term ---------------------------------- */

#[enum_dispatch(ValueT)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermValue {
    TermAnn(Ann<TermAnn<TV, T>>),
    Var(Ann<TermV>),
    Thunk(Ann<Thunk<TC>>),
    Ctor(Ann<Ctor<Ann<CtorV>, TV>>),
}
type TV = Rc<TermValue>;
impl ValueT for TermValue {}

#[enum_dispatch(ComputationT)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermComputation {
    TermAnn(Ann<TermAnn<TC, T>>),
    Ret(Ann<Ret<TV>>),
    Force(Ann<Force<TV>>),
    Let(Ann<Let<Ann<TermV>, TV, TC>>),
    Do(Ann<Do<Ann<TermV>, TC>>),
    Rec(Ann<Rec<Ann<TermV>, TC>>),
    Match(Ann<Match<Ann<TermV>, TV, TC>>),
    CoMatch(Ann<CoMatch<Ann<TermV>, TC>>),
    Dtor(Ann<Dtor<Ann<DtorV>, TC, TV>>),
}
type TC = Rc<TermComputation>;
impl ComputationT for TermComputation {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Val(TermValue),
    Comp(TermComputation),
}

/* --------------------------------- Module --------------------------------- */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Module {
    pub name: Option<String>,
    pub type_ctx: HashMap<Ann<TypeV>, TypeArity<Kind>>,
    pub data: Vec<Ann<Data<Ann<TermV>, Ann<CtorV>, T>>>,
    pub codata: Vec<Ann<Codata<Ann<TermV>, Ann<DtorV>, T>>>,
    pub define: Vec<Ann<Define<Ann<TermV>, T, TV>>>,
    pub entry: Ann<TermComputation>,
}

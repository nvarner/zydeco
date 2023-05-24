use crate::prelude::*;
use im::Vector;
use std::rc::Rc;
use zydeco_derive::{FmtArgs, IntoEnum};

pub use crate::syntax::*;

/* ---------------------------------- Kind ---------------------------------- */

pub use crate::syntax::{KindBase, TypeArity};

#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum Kind {
    Base(KindBase),
    TypeArity(TypeArity<Span<Kind>, BoxKind>),
}
pub type BoxKind = Box<Span<Kind>>;
impl KindT for Kind {}

/* ---------------------------------- Type ---------------------------------- */

#[derive(Clone, Debug, PartialEq)]
pub struct AbstVar(pub usize);
#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum SynType {
    TypeAbs(TypeAbs<(TypeV, Span<Kind>), RcType>), 
    TypeApp(TypeApp<TypeV, RcType>),
    Forall(Forall<(TypeV, Span<Kind>), RcType>),
    Exists(Exists<(TypeV, Span<Kind>), RcType>),
    AbstVar(AbstVar),
    Hole(Hole),
}

#[derive(Clone, Debug)]
pub struct Type {
    pub synty: SynType,
}
pub type RcType = Rc<Span<Type>>;
impl TypeT for Type {}

macro_rules! impl_from {
    ($T:ty) => {
        impl From<$T> for Type {
            fn from(synty: $T) -> Self {
                Self { synty: synty.into() }
            }
        }
    };
}
impl_from!(TypeAbs<(TypeV, Span<Kind>), RcType>);
impl_from!(TypeApp<TypeV, RcType>);
impl_from!(Forall<(TypeV, Span<Kind>), RcType>);
impl_from!(Exists<(TypeV, Span<Kind>), RcType>);
impl_from!(AbstVar);
impl_from!(Hole);
impl From<TypeV> for Type {
    fn from(tvar: TypeV) -> Self {
        TypeApp { tvar, args: vec![] }.into()
    }
}

/* ---------------------------------- Term ---------------------------------- */

#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum TermValue {
    Annotation(Annotation<RcValue, RcType>),
    Var(TermV),
    Thunk(Thunk<RcComp>),
    Ctor(Ctor<CtorV, RcValue>),
    Literal(Literal),
    Pack(Pack<RcType, RcValue>),
}
pub type RcValue = Rc<Span<TermValue>>;
impl ValueT for TermValue {}

#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum TailTerm {
    Let(Let<TermV, RcValue, ()>),
    Do(Do<TermV, RcComp, ()>),
}

#[derive(Clone, Debug)]
pub struct TailGroup {
    pub group: Vector<TailTerm>,
    pub body: RcComp,
}

#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum TermComputation {
    Annotation(Annotation<RcComp, RcType>),
    Ret(Ret<RcValue>),
    Force(Force<RcValue>),
    TailGroup(TailGroup),
    Rec(Rec<TermV, RcComp>),
    Match(Match<CtorV, TermV, RcValue, RcComp>),
    Comatch(Comatch<DtorV, TermV, RcComp>),
    Dtor(Dtor<RcComp, DtorV, RcValue>),
    TyAbsTerm(Abs<(TypeV, Option<Span<Kind>>), RcComp>),
    TyAppTerm(App<RcComp, RcType>),
    MatchPack(MatchPack<RcValue, TypeV, TermV, RcComp>),
}
pub type RcComp = Rc<Span<TermComputation>>;
impl ComputationT for TermComputation {}

#[derive(IntoEnum, FmtArgs, Clone, Debug)]
pub enum Term {
    Value(TermValue),
    Computation(TermComputation),
}

/* --------------------------------- Module --------------------------------- */

#[derive(Clone, Debug)]
pub struct Module {
    pub name: Option<String>,
    pub data: Vec<DeclSymbol<prelude::Data>>,
    pub codata: Vec<DeclSymbol<prelude::Codata>>,
    pub alias: Vec<DeclSymbol<prelude::Alias>>,
    pub define: Vec<DeclSymbol<Define<TermV, RcValue>>>,
    pub define_ext: Vec<DeclSymbol<Define<(TermV, RcType), ()>>>,
}

#[derive(Clone, Debug)]
pub struct Program {
    pub module: Span<Module>,
    pub entry: Span<TermComputation>,
}

pub mod prelude {
    use super::*;
    pub type Data = super::Data<TypeV, Span<Kind>, CtorV, RcType>;
    pub type Codata = super::Codata<TypeV, Span<Kind>, DtorV, RcType>;
    pub type Alias = super::Alias<TypeV, Span<Kind>, RcType>;
}

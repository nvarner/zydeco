pub mod env;
mod fmt;
mod span;

use zydeco_derive::IntoEnum;

/* ---------------------------------- Meta ---------------------------------- */

pub mod sort {
    use crate::utils::span::Span;
    use std::rc::Rc;

    macro_rules! sort {
        ( $Sort:ident ) => {
            pub trait $Sort {}
            impl<T: $Sort> $Sort for Box<T> {}
            impl<T: $Sort> $Sort for Rc<T> {}
            impl<T: $Sort> $Sort for Span<T> {}
            impl<T: $Sort> $Sort for Option<T> {}
            impl $Sort for () {}
        };
    }

    sort!(VarT);
    sort!(TyVarT);
    sort!(CtorT);
    sort!(DtorT);
    sort!(KindT);
    sort!(TypeT);
    sort!(ValueT);
    sort!(ComputationT);
}
pub use sort::*;

/* --------------------------------- Binders -------------------------------- */

pub mod binder {
    use super::sort::*;
    use crate::utils::span::{Span, SpanHolder, SpanInfo, SpanView};
    use std::{
        cmp::{Eq, PartialEq},
        hash::{Hash, Hasher},
    };

    macro_rules! var {
        ( $Var:ident ) => {
            #[allow(clippy::mutable_key_type)]
            #[derive(Clone, Debug)]
            pub struct $Var<Id = String, Ty = ()> {
                name: Id,
                info: SpanInfo,
                #[allow(unused)]
                ty: Ty,
            }
            impl<Id: AsRef<str> + Eq, Ty: Default> $Var<Id, Ty> {
                pub fn new(name: Id, info: SpanInfo) -> Self {
                    Self { name, info, ty: Default::default() }
                }
                pub fn name(&self) -> &str {
                    self.name.as_ref()
                }
            }
            impl From<Span<String>> for $Var {
                fn from(span: Span<String>) -> Self {
                    Self { name: span.inner, info: span.info, ty: Default::default() }
                }
            }
            impl PartialEq for $Var {
                fn eq(&self, other: &Self) -> bool {
                    self.name.eq(&other.name)
                }
            }
            impl Eq for $Var {}
            impl Hash for $Var {
                fn hash<H: Hasher>(&self, state: &mut H) {
                    self.name.hash(state);
                }
            }
            impl SpanView for $Var {
                fn span(&self) -> &SpanInfo {
                    &self.info
                }
            }
            impl SpanHolder for $Var {
                fn span_map_mut<F>(&mut self, f: F)
                where
                    F: Fn(&mut SpanInfo) + Clone,
                {
                    f(&mut self.info);
                }
            }
        };
    }

    var!(CtorV);
    impl CtorT for CtorV {}
    var!(DtorV);
    impl DtorT for DtorV {}
    var!(TypeV);
    impl TyVarT for TypeV {}
    impl<Kd: KindT> TyVarT for (TypeV, Kd) {}
    var!(TermV);
    impl VarT for TermV {}
    impl<Ty: TypeT> VarT for (TermV, Ty) {}
    var!(ModV);
    impl VarT for ModV {}
}
pub use binder::*;

/* ------------------------------ Bi-Diretional ----------------------------- */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Annotation<Term, Type> {
    pub term: Term,
    pub ty: Type,
}
impl<Term: ValueT, Type> ValueT for Annotation<Term, Type> {}
impl<Term: ComputationT, Type> ComputationT for Annotation<Term, Type> {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Hole;
impl TypeT for Hole {}

/* ---------------------------------- Kind ---------------------------------- */

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KindBase {
    VType,
    CType,
}
impl KindT for KindBase {}

/// A kind that represents the arity, a.k.a. parameters of a type constructor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeArity<In: KindT, Out: KindT> {
    pub params: Vec<In>,
    pub kd: Out,
}
impl<In: KindT, Out: KindT> KindT for TypeArity<In, Out> {}

/* ---------------------------------- Types --------------------------------- */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeApp<TyV: TyVarT, Ty: TypeT> {
    pub tvar: TyV,
    pub args: Vec<Ty>,
}
impl<TyV: TyVarT, Ty: TypeT> TypeT for TypeApp<TyV, Ty> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Forall<TyV: TyVarT, Ty: TypeT> {
    pub param: TyV,
    pub ty: Ty,
}
impl<TyV: TyVarT, Ty: TypeT> TypeT for Forall<TyV, Ty> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exists<TyV: TyVarT, Ty: TypeT> {
    pub param: TyV,
    pub ty: Ty,
}
impl<TyV: TyVarT, Ty: TypeT> TypeT for Exists<TyV, Ty> {}

/* --------------------------------- Values --------------------------------- */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thunk<B: ComputationT>(pub B);
impl<B: ComputationT> ValueT for Thunk<B> {}

#[derive(IntoEnum, Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Int(i64),
    String(String),
    Char(char),
}
impl ValueT for Literal {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ctor<C: CtorT, A: ValueT> {
    pub ctor: C,
    pub args: Vec<A>,
}
impl<C: CtorT, A: ValueT> ValueT for Ctor<C, A> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pack<Ty: TypeT, A: ValueT> {
    pub ty: Ty,
    pub body: A,
}
impl<Ty: TypeT, A: ValueT> ValueT for Pack<Ty, A> {}

/* ------------------------------ Computations ------------------------------ */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ret<A: ValueT>(pub A);
impl<A: ValueT> ComputationT for Ret<A> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Force<A: ValueT>(pub A);
impl<A: ValueT> ComputationT for Force<A> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Let<TeV: VarT, A: ValueT, B: ComputationT> {
    pub var: TeV,
    pub def: A,
    pub body: B,
}
impl<TeV: VarT, A: ValueT, B: ComputationT> ComputationT for Let<TeV, A, B> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Do<TeV: VarT, B1: ComputationT, B2: ComputationT> {
    pub var: TeV,
    pub comp: B1,
    pub body: B2,
}
impl<TeV: VarT, B1: ComputationT, B2: ComputationT> ComputationT for Do<TeV, B1, B2> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rec<TeV: VarT, B: ComputationT> {
    pub var: TeV,
    pub body: B,
}
impl<TeV: VarT, B: ComputationT> ComputationT for Rec<TeV, B> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match<C: CtorT, TeV: VarT, A: ValueT, B: ComputationT> {
    pub scrut: A,
    pub arms: Vec<Matcher<C, TeV, B>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matcher<C: CtorT, TeV: VarT, B: ComputationT> {
    pub ctor: C,
    pub vars: Vec<TeV>,
    pub body: B,
}
impl<C: CtorT, TeV: VarT, A: ValueT, B: ComputationT> ComputationT for Match<C, TeV, A, B> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comatch<D: DtorT, TeV: VarT, B: ComputationT> {
    pub arms: Vec<Comatcher<D, TeV, B>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comatcher<D: DtorT, TeV: VarT, B: ComputationT> {
    pub dtor: D,
    pub vars: Vec<TeV>,
    pub body: B,
}
impl<D: DtorT, TeV: VarT, B: ComputationT> ComputationT for Comatch<D, TeV, B> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dtor<B: ComputationT, D: DtorT, A: ValueT> {
    pub body: B,
    pub dtor: D,
    pub args: Vec<A>,
}
impl<B: ComputationT, D: DtorT, A: ValueT> ComputationT for Dtor<B, D, A> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyAbsTerm<TyV: TyVarT, B: ComputationT> {
    pub param: TyV,
    pub body: B,
}
impl<TyV: TyVarT, B: ComputationT> ComputationT for TyAbsTerm<TyV, B> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TyAppTerm<B: ComputationT, Ty: TypeT> {
    pub body: B,
    pub arg: Ty,
}
impl<B: ComputationT, Ty: TypeT> ComputationT for TyAppTerm<B, Ty> {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchPack<A: ValueT, TyV: TyVarT, TeV: VarT, B: ComputationT> {
    pub scrut: A,
    pub tvar: TyV,
    pub var: TeV,
    pub body: B,
}
impl<A: ValueT, TyV: TyVarT, TeV: VarT, B: ComputationT> ComputationT
    for MatchPack<A, TyV, TeV, B>
{
}

/* ------------------------------ Declarations ------------------------------ */

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclSymbol<T> {
    pub public: bool,
    pub external: bool,
    pub inner: T,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Data<TyV: TyVarT, Kd: KindT, C: CtorT, Ty: TypeT> {
    pub name: TyV,
    pub params: Vec<(TyV, Kd)>,
    pub ctors: Vec<DataBr<C, Ty>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataBr<C: CtorT, Ty: TypeT>(pub C, pub Vec<Ty>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Codata<TyV: TyVarT, Kd: KindT, D: DtorT, Ty: TypeT> {
    pub name: TyV,
    pub params: Vec<(TyV, Kd)>,
    pub dtors: Vec<CodataBr<D, Ty>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodataBr<D: DtorT, Ty: TypeT>(pub D, pub Vec<Ty>, pub Ty);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Alias<TyV: TyVarT, Kd: KindT, Ty: TypeT> {
    pub name: TyV,
    pub params: Vec<(TyV, Kd)>,
    pub ty: Ty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Define<TeV: VarT, A: ValueT> {
    pub name: TeV,
    pub def: A,
}

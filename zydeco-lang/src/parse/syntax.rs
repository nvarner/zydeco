use crate::utils::ann::Ann;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub decls: Vec<Declare>,
    pub comp: Box<Compute>,
    pub ann: Ann,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValOrComp {
    Val(Value),
    Comp(Compute),
}

pub type Binding<Ty, Def> = (VVar, Option<Box<Ty>>, Box<Def>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declare {
    Data {
        name: TVar,
        ctors: Vec<(Ctor, Vec<Type>)>,
        ann: Ann,
    },
    Codata {
        name: TVar,
        dtors: Vec<(Dtor, Vec<Type>, Type)>,
        ann: Ann,
    },
    Define {
        public: bool,
        name: VVar,
        ty: Option<Box<Type>>,
        def: Option<Box<Value>>,
        ann: Ann,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Var(VVar, Ann),
    Thunk(Box<Compute>, Ann),
    Ctor(Ctor, Vec<Value>, Ann),
    Int(i64, Ann),
    String(String, Ann),
    Char(char, Ann),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Compute {
    Let {
        binding: Binding<Type, Value>,
        body: Box<Compute>,
        ann: Ann,
    },
    Do {
        binding: Binding<Type, Compute>,
        body: Box<Compute>,
        ann: Ann,
    },
    Force(Box<Value>, Ann),
    Return(Box<Value>, Ann),
    Lam {
        arg: (VVar, Option<Box<Type>>),
        body: Box<Compute>,
        ann: Ann,
    },
    Rec {
        arg: (VVar, Option<Box<Type>>),
        body: Box<Compute>,
        ann: Ann,
    },
    App(Box<Compute>, Box<Value>, Ann),
    Match {
        scrut: Box<Value>,
        cases: Vec<(Ctor, Vec<VVar>, Box<Compute>)>,
        ann: Ann,
    },
    CoMatch {
        cases: Vec<(Dtor, Vec<VVar>, Box<Compute>)>,
        ann: Ann,
    },
    CoApp {
        body: Box<Compute>,
        dtor: Dtor,
        args: Vec<Value>,
        ann: Ann,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Var(TVar, Ann),
    Thunk(Box<Type>, Ann),
    Ret(Box<Type>, Ann),
    Lam(Box<Type>, Box<Type>, Ann),
    OS,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    ValType,
    CompType,
}

macro_rules! var {
    ( $Var:ident ) => {
        #[derive(Clone, Debug)]
        pub struct $Var(String, Ann);
        impl $Var {
            pub fn new(s: String, ann: Ann) -> Self {
                Self(s, ann)
            }
            pub fn name(&self) -> &str {
                &self.0
            }
            pub fn ann(&self) -> &Ann {
                &self.1
            }
        }
        impl std::cmp::PartialEq for $Var {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl std::cmp::Eq for $Var {}
        impl std::hash::Hash for $Var {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.0.hash(state);
            }
        }
    };
}

var!(Ctor);
var!(Dtor);
var!(TVar);
var!(VVar);

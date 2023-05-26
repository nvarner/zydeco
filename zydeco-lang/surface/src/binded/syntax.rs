use crate::textual::syntax::*;
use derive_more::From;
use slotmap::SlotMap;
// use std::collections::{HashMap, HashSet};

#[derive(From)]
pub enum Declaration {
    Type(TypeDef),
    Define(Define),
    Main(Main),
}

// slotmap::new_key_type! {
//     pub struct TopId;
// }

// pub struct TopSlice {
//     pub deps: HashSet<TopId>,
//     pub decls: Vec<Declaration>,
// }
// pub struct TopLevel {
//     pub map: SlotMap<TopId, TopSlice>,
//     pub tops: HashSet<TopId>,
//     pub blocks: HashMap<TopId, HashSet<TopId>>,
// }
pub type TopLevel = Vec<Declaration>;

pub struct Context {
    // arenas
    pub patterns: SlotMap<PatternId, Sp<Pattern>>,
    pub terms: SlotMap<TermId, Sp<Term<DefId>>>,
    // meta
    pub lookup: im::HashMap<NameRef<VarName>, DefId>,
    pub peeks: im::HashMap<NameRef<VarName>, DefId>,
}

impl Context {
    pub fn pattern(&mut self, pattern: Sp<Pattern>) -> PatternId {
        self.patterns.insert(pattern)
    }
    pub fn term(&mut self, term: Sp<Term<DefId>>) -> TermId {
        self.terms.insert(term)
    }
}

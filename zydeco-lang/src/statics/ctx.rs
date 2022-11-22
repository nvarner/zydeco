use super::{err::TypeCheckError, resolve::*, tyck::TypeCheck};
use crate::{parse::syntax::*, utils::ann::AnnT};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Sort {
    TVal,
    TComp,
}

#[derive(Clone, Debug)]
pub struct Ctx<Ann> {
    vmap: HashMap<VVar<Ann>, TValue<Ann>>,
    pub tmap: HashMap<TVar<Ann>, Sort>,
    data: HashMap<TVar<Ann>, Vec<(Ctor<Ann>, Vec<TValue<Ann>>)>>,
    pub ctors: HashMap<Ctor<Ann>, (TVar<Ann>, Vec<TValue<Ann>>)>,
    codata:
        HashMap<TVar<Ann>, Vec<(Dtor<Ann>, Vec<TValue<Ann>>, TCompute<Ann>)>>,
    pub dtors: HashMap<Dtor<Ann>, (TVar<Ann>, Vec<TValue<Ann>>, TCompute<Ann>)>,
}

impl<Ann: AnnT> Ctx<Ann> {
    pub fn new() -> Self {
        Self {
            vmap: HashMap::new(),
            tmap: HashMap::new(),
            data: HashMap::new(),
            ctors: HashMap::new(),
            codata: HashMap::new(),
            dtors: HashMap::new(),
        }
    }
    pub fn push(&mut self, x: VVar<Ann>, t: TValue<Ann>) {
        self.vmap.insert(x, t);
    }
    pub fn extend(
        &mut self, other: impl IntoIterator<Item = (VVar<Ann>, TValue<Ann>)>,
    ) {
        self.vmap.extend(other);
    }
    pub fn lookup(&self, x: &VVar<Ann>) -> Option<&TValue<Ann>> {
        self.vmap.get(x)
    }
    pub fn decl(
        &mut self, d: &Declare<Ann>,
    ) -> Result<(), NameResolveError<Ann>> {
        match d {
            Declare::Data { name, ctors, ann } => {
                self.data.insert(name.clone(), ctors.clone()).map_or(
                    Ok(()),
                    |_| {
                        Err(NameResolveError::DuplicateDeclaration {
                            name: name.name().to_string(),
                            ann: ann.clone(),
                        })
                    },
                )?;
                self.tmap.insert(name.clone(), Sort::TVal);
                for (ctor, args) in ctors {
                    self.ctors
                        .insert(ctor.clone(), (name.clone(), args.clone()))
                        .map_or(Ok(()), |_| {
                            Err(NameResolveError::DuplicateDeclaration {
                                name: ctor.name().to_string(),
                                ann: ann.clone(),
                            })
                        })?;
                }
                Ok(())
            }
            Declare::Codata { name, dtors, ann } => {
                self.codata.insert(name.clone(), dtors.clone()).map_or(
                    Ok(()),
                    |_| {
                        Err(NameResolveError::DuplicateDeclaration {
                            name: name.name().to_string(),
                            ann: ann.clone(),
                        })
                    },
                )?;
                self.tmap.insert(name.clone(), Sort::TComp);
                for (dtor, args, ret) in dtors {
                    self.dtors
                        .insert(
                            dtor.clone(),
                            (name.clone(), args.clone(), ret.clone()),
                        )
                        .map_or(Ok(()), |_| {
                            Err(NameResolveError::DuplicateDeclaration {
                                name: dtor.name().to_string(),
                                ann: ann.clone(),
                            })
                        })?;
                }
                Ok(())
            }
            Declare::Define { name, ty: Some(ty), .. } => {
                self.push(name.clone(), *ty.to_owned());
                Ok(())
            }
            _ => panic!("define in module is not unimplemented"),
        }
    }
    pub fn tyck(&self) -> Result<(), TypeCheckError<Ann>> {
        for (_, ctors) in &self.data {
            for (_, args) in ctors {
                for arg in args {
                    arg.tyck(self)?;
                }
            }
        }
        for (_, dtors) in &self.codata {
            for (_, args, ret) in dtors {
                for arg in args {
                    arg.tyck(self)?;
                }
                ret.tyck(self)?;
            }
        }
        Ok(())
    }
}

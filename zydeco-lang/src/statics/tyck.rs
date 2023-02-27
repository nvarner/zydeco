use super::{ctx::*, err::TypeCheckError, resolve::NameResolveError};
use crate::{
    parse::syntax::*,
    syntax::ann::{ann, Ann, AnnHolder, AnnInfo},
    syntax::binder::*,
};
use std::collections::HashMap;
use TypeCheckError::*;

pub trait TypeCheck {
    type Out: Eqv;
    fn syn(&self, ctx: &Ctx) -> Result<Self::Out, Ann<TypeCheckError>>;
    fn ana(
        &self, typ: &Self::Out, ctx: &Ctx,
    ) -> Result<(), Ann<TypeCheckError>> {
        let typ_syn = self.syn(ctx)?;
        typ.eqv(&typ_syn).ok_or_else(|| {
            ann(0, 0).make(ErrStr(format!("Subsumption failed")))
        })
    }
}

impl TypeCheck for Program {
    type Out = ();

    fn syn(&self, ctx: &Ctx) -> Result<Self::Out, Ann<TypeCheckError>> {
        let mut ctx = ctx.clone();
        for decl in &self.decls {
            ctx.decl(decl).map_err(|e| self.ann.make(NameResolve(e)))?;
        }
        ctx.type_validation()?;
        ctx.tyck_definitions()?;
        let typ = self.comp.syn(&ctx)?;
        match &typ.ctor {
            TCtor::OS => Ok(()),
            _ => Err(self.ann.make(WrongMain { found: typ })),
        }
    }
}

impl Kind {
    fn ensure_vtype(
        &self, context: &str, ann: &AnnInfo,
    ) -> Result<(), Ann<TypeCheckError>> {
        if let Kind::CType = self {
            Err(ann.make(TypeCheckError::KindMismatch {
                context: context.to_owned(),
                expected: Kind::VType,
                found: *self,
            }))
        } else {
            Ok(())
        }
    }
    fn expect_comptype(
        &self, context: &str, ann: &AnnInfo,
    ) -> Result<(), Ann<TypeCheckError>> {
        if let Kind::VType = self {
            Err(ann.make(TypeCheckError::KindMismatch {
                context: context.to_owned(),
                expected: Kind::CType,
                found: *self,
            }))
        } else {
            Ok(())
        }
    }
}

impl TypeCheck for Value {
    type Out = Type;
    fn syn(&self, ctx: &Ctx) -> Result<Self::Out, Ann<TypeCheckError>> {
        match self {
            Value::TermAnn(body, typ, ..) => {
                body.ana(typ, ctx)?;
                Ok(typ.clone())
            }
            Value::Var(x, ann) => ctx
                .lookup(x)
                .cloned()
                .ok_or(ann.make(UnboundVar { var: x.clone() })),
            Value::Thunk(e, ann) => {
                let t = e.syn(&ctx)?;
                Ok(Type { ctor: TCtor::Thunk, args: vec![t], ann: ann.clone() })
            }
            Value::Ctor(_, _, ann) => {
                Err(ann.make(NeedAnnotation { content: format!("ctor") }))
            }
            Value::Int(_, ann) => {
                Ok(Type::internal("Int", vec![], ann.clone()))
            }
            Value::String(_, ann) => {
                Ok(Type::internal("String", vec![], ann.clone()))
            }
            Value::Char(_, ann) => {
                Ok(Type::internal("Char", vec![], ann.clone()))
            }
        }
    }

    fn ana(
        &self, typ: &Self::Out, ctx: &Ctx,
    ) -> Result<(), Ann<TypeCheckError>> {
        match self {
            Value::Thunk(e, ann, ..) => {
                if let TCtor::Thunk = typ.ctor {
                    if typ.args.len() != 1 {
                        return Err(ann.make(ArityMismatch {
                            context: format!("thunk"),
                            expected: 1,
                            found: typ.args.len(),
                        }));
                    }
                    e.ana(&typ.args[0], ctx)
                } else {
                    Err(ann.make(TypeExpected {
                        expected: format!("thunk"),
                        found: typ.to_owned(),
                    }))
                }
            }
            Value::Ctor(ctor, args, ann, ..) => {
                if let TCtor::Var(tvar) = &typ.ctor {
                    let data = ctx.data.get(tvar).ok_or_else(|| {
                        ann.make(ErrStr(format!("unknown ctor: {}", tvar)))
                    })?;
                    let targs = data
                        .ctors
                        .iter()
                        .find(|(ctor_, _)| ctor == ctor_)
                        .ok_or_else(|| {
                            ann.make(ErrStr(format!("unknown ctor: {}", ctor)))
                        })?
                        .1
                        .clone();
                    if args.len() != targs.len() {
                        return Err(ann.make(ArityMismatch {
                            context: format!(
                                "application of constructor {}",
                                ctor
                            ),
                            expected: targs.len(),
                            found: args.len(),
                        }));
                    }
                    for (arg, targ) in args.iter().zip(targs.iter()) {
                        arg.ana(&targ, ctx)?;
                    }
                    Ok(())
                } else {
                    Err(ann.make(TypeExpected {
                        expected: format!("constructor {}", ctor),
                        found: typ.to_owned(),
                    }))
                }
            }
            v => {
                let t = self.syn(ctx)?;
                typ.eqv(&t).ok_or_else(|| {
                    v.ann()
                        .make(TypeMismatch { expected: typ.clone(), found: t })
                })
            }
        }
    }
}

impl TypeCheck for Compute {
    type Out = Type;

    fn syn(&self, ctx: &Ctx) -> Result<Self::Out, Ann<TypeCheckError>> {
        match self {
            Compute::TermAnn(body, ty, ..) => {
                body.ana(ty, ctx)?;
                Ok(ty.clone())
            }
            Compute::Let { binding: (x, _, def), body, .. } => {
                let mut ctx = ctx.clone();
                let t = def.syn(&ctx)?;
                ctx.push(x.clone(), t);
                body.syn(&ctx)
            }
            Compute::Do { binding: (x, _, def), body, ann, .. } => {
                let mut ctx = ctx.clone();
                let te = def.syn(&ctx)?;
                match te.ctor {
                    TCtor::Ret => {
                        ctx.push(x.clone(), te.args[0].clone());
                        body.syn(&ctx)
                    }
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("Ret(a?)"),
                        found: te.into(),
                    })),
                }
            }
            Compute::Force(comp, ann, ..) => {
                let t = comp.syn(&ctx)?;
                match t.ctor {
                    TCtor::Thunk => Ok(t.args[0].clone()),
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("Thunk(b?)"),
                        found: t.into(),
                    })),
                }
            }
            Compute::Return(v, ann, ..) => {
                let t = v.syn(&ctx)?;
                Ok(Type { ctor: TCtor::Ret, args: vec![t], ann: ann.clone() })
            }
            Compute::Lam { arg: (x, t), body, ann } => {
                let mut ctx = ctx.clone();
                let t = t.as_ref().ok_or_else(|| {
                    ann.make(NeedAnnotation {
                        content: format!("lambda parameter \"{}\"", x),
                    })
                })?;
                t.syn(&ctx)?.ensure_vtype("argument to a function", ann)?;
                ctx.push(x.clone(), *t.clone());
                let tbody = body.syn(&ctx)?;
                Ok(Type {
                    ctor: TCtor::Fun,
                    args: vec![*t.clone(), tbody],
                    ann: ann.clone(),
                })
            }
            Compute::Rec { arg: (x, ty), body, ann, .. } => {
                let mut ctx = ctx.clone();
                let ty = ty.as_ref().ok_or_else(|| {
                    ann.make(NeedAnnotation {
                        content: format!("recursive computation \"{}\"", x),
                    })
                })?;
                // don't need to check this is a value type bc we check it's a thunk next
                ty.syn(&ctx)?;
                let ty_body = match ty.ctor {
                    TCtor::Thunk => ty.args[0].clone(),
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("Thunk(b?)"),
                        found: ty.as_ref().to_owned().into(),
                    }))?,
                };
                ctx.push(x.clone(), *ty.clone());
                let tbody_ = body.syn(&ctx)?;
                ty_body.eqv(&tbody_).ok_or_else(|| {
                    ann.make(TypeMismatch {
                        expected: ty_body.clone().into(),
                        found: tbody_.into(),
                    })
                })?;
                Ok(ty_body)
            }
            Compute::App(e, v, ann) => {
                let tfn = e.syn(&ctx)?;
                let targ = v.syn(&ctx)?;
                match tfn.ctor {
                    TCtor::Fun => {
                        let ty_dom = tfn.args[0].clone();
                        let ty_cod = tfn.args[1].clone();
                        Type::eqv(&ty_dom, &targ).ok_or_else(|| {
                            ann.make(TypeMismatch {
                                expected: ty_dom.to_owned().into(),
                                found: targ.into(),
                            })
                        })?;
                        Ok(ty_cod)
                    }
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("a? -> b?"),
                        found: tfn.into(),
                    })),
                }
            }
            Compute::Match { scrut, arms, ann, .. } => {
                let scrut_ty = scrut.syn(&ctx)?;
                let TCtor::Var(data_ty_name) = scrut_ty.ctor else {
                    Err(ann.make(TypeExpected {
                        expected: format!("a?"),
                        found: scrut_ty.into(),
                    }))?
                };
                let data = ctx.data.get(&data_ty_name).ok_or_else(|| {
                    ann.make(ErrStr(format!(
                        "unknown data type: {}",
                        data_ty_name
                    )))
                })?;
                let mut ctors: HashMap<CtorV, Vec<Type>> =
                    HashMap::from_iter(data.ctors.clone());
                let mut ty = None;
                for (ctor, vars, body) in arms {
                    let Some(ty_args) = ctors.remove(ctor) else {
                        Err(ann.make(ErrStr(format!("unknown ctor: {}", ctor))))?
                    };
                    // check if the ctor has the right number of arguments
                    if vars.len() != ty_args.len() {
                        return Err(ann.make(ArityMismatch {
                            context: format!("`match` arm for {}", ctor),
                            expected: vars.len(),
                            found: ty_args.len(),
                        }));
                    }
                    // check the body of the branch
                    let mut ctx = ctx.clone();
                    ctx.extend(
                        vars.iter().cloned().zip(ty_args.iter().cloned()),
                    );
                    if let Some(ty) = &ty {
                        body.ana(ty, &ctx)?;
                    } else {
                        ty = Some(body.syn(&ctx)?);
                    }
                }
                // check that all ctors were covered
                if !ctors.is_empty() {
                    return Err(ann.make(ErrStr(format!(
                        "{} uncovered ctors",
                        ctors.len()
                    ))));
                }
                let Some(ty) = ty else {
                    Err(ann.make(NeedAnnotation { content: format!("empty match") }))?
                };
                Ok(ty)
            }
            Compute::CoApp { body, dtor, args, ann, .. } => {
                let tscrut = body.syn(ctx)?;
                if let TCtor::Var(tvar) = tscrut.ctor {
                    let coda = ctx.coda.get(&tvar).ok_or_else(|| {
                        ann.make(ErrStr(format!("unknown codata: {}", tvar)))
                    })?;
                    let (_, ty_args, tret) = coda
                        .dtors
                        .iter()
                        .find(|(dtor_, _, _)| dtor == dtor_)
                        .ok_or_else(|| {
                            ann.make(ErrStr(format!("unknown dtor: {}", dtor)))
                        })?;
                    if args.len() != ty_args.len() {
                        return Err(ann.make(ArityMismatch {
                            context: format!(
                                "application of destructor {}",
                                dtor
                            ),
                            expected: ty_args.len(),
                            found: args.len(),
                        }));
                    }
                    for (arg, expected) in args.iter().zip(ty_args.iter()) {
                        arg.ana(expected, ctx)?;
                    }
                    Ok(tret.clone())
                } else {
                    Err(ann.make(TypeExpected {
                        expected: format!("a?"),
                        found: tscrut.into(),
                    }))
                }
            }
            Compute::CoMatch { ann, .. } => {
                Err(ann.make(NeedAnnotation { content: format!("comatch") }))
            }
        }
    }

    fn ana(
        &self, typ: &Self::Out, ctx: &Ctx,
    ) -> Result<(), Ann<TypeCheckError>> {
        match self {
            Compute::Let { binding: (x, _, def), body, .. } => {
                let mut ctx = ctx.clone();
                let t = def.syn(&ctx)?;
                ctx.push(x.clone(), t);
                body.ana(typ, &ctx)
            }
            Compute::Do { binding: (x, _, def), body, ann, .. } => {
                let mut ctx = ctx.clone();
                let te = def.syn(&ctx)?;
                match te.ctor {
                    TCtor::Ret => {
                        ctx.push(x.clone(), te.args[0].clone());
                        body.ana(typ, &ctx)
                    }
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("Ret(a?)"),
                        found: te.into(),
                    })),
                }
            }
            Compute::Force(comp, ..) => comp.ana(typ, ctx),
            Compute::Return(v, ann, ..) => {
                let t = v.syn(&ctx)?;
                typ.eqv(&t).ok_or_else(|| {
                    ann.make(TypeMismatch {
                        expected: typ.to_owned(),
                        found: t,
                    })
                })
            }
            Compute::Lam { arg: (x, t), body, ann, .. } => {
                let mut ctx = ctx.clone();
                let t = t.as_ref().ok_or_else(|| {
                    ann.make(NeedAnnotation {
                        content: format!("lambda parameter \"{}\"", x),
                    })
                })?;
                t.syn(&ctx)?.ensure_vtype("argument to a function", ann)?;
                ctx.push(x.clone(), *t.clone());
                let tbody = body.syn(&ctx)?;
                typ.eqv(&tbody).ok_or_else(|| {
                    ann.make(TypeMismatch {
                        expected: typ.to_owned(),
                        found: tbody,
                    })
                })
            }
            Compute::Rec { arg: (x, ty), body, ann, .. } => {
                let mut ctx = ctx.clone();
                let ty = ty.as_ref().ok_or_else(|| {
                    ann.make(NeedAnnotation {
                        content: format!("recursive computation \"{}\"", x),
                    })
                })?;
                // don't need to check this is a value type bc we check it's a thunk next
                ty.syn(&ctx)?;
                let ty_body = match ty.ctor {
                    TCtor::Thunk => ty.args[0].clone(),
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("Thunk(b?)"),
                        found: ty.as_ref().to_owned().into(),
                    }))?,
                };
                Type::eqv(&ty_body, typ).ok_or_else(|| {
                    ann.make(TypeMismatch {
                        expected: typ.to_owned(),
                        found: ty_body,
                    })
                })?;
                ctx.push(x.clone(), *ty.clone());
                body.ana(typ, &ctx)
            }
            Compute::App(e, v, ann) => {
                let tfn = e.syn(&ctx)?;
                match tfn.ctor {
                    TCtor::Fun => {
                        let ty_cod = tfn.args[1].clone();
                        Type::eqv(&ty_cod, &typ).ok_or_else(|| {
                            ann.make(TypeMismatch {
                                expected: typ.to_owned().into(),
                                found: ty_cod.into(),
                            })
                        })?;
                        let ty_dom = tfn.args[0].clone();
                        v.ana(&ty_dom, ctx)
                    }
                    _ => Err(ann.make(TypeExpected {
                        expected: format!("a? -> b?"),
                        found: tfn.into(),
                    })),
                }
            }
            Compute::Match { scrut, arms, ann, .. } => {
                let scrut_ty = scrut.syn(&ctx)?;
                let TCtor::Var(data_ty_name) = scrut_ty.ctor else {
                    Err(ann.make(TypeExpected {
                        expected: format!("a?"),
                        found: scrut_ty.into(),
                    }))?
                };
                let data = ctx.data.get(&data_ty_name).ok_or_else(|| {
                    ann.make(ErrStr(format!(
                        "unknown data type: {}",
                        data_ty_name
                    )))
                })?;
                let mut ctors: HashMap<CtorV, Vec<Type>> =
                    HashMap::from_iter(data.ctors.clone());
                for (ctor, vars, body) in arms {
                    let Some(ty_args) = ctors.remove(ctor) else {
                        Err(ann.make(ErrStr(format!("unknown ctor: {}", ctor))))?
                    };
                    // check if the ctor has the right number of arguments
                    if vars.len() != ty_args.len() {
                        return Err(ann.make(ArityMismatch {
                            context: format!("`match` arm for {}", ctor),
                            expected: vars.len(),
                            found: ty_args.len(),
                        }));
                    }
                    // check the body of the branch
                    let mut ctx = ctx.clone();
                    ctx.extend(
                        vars.iter().cloned().zip(ty_args.iter().cloned()),
                    );
                    body.ana(typ, &ctx)?;
                }
                // check that all ctors were covered
                if !ctors.is_empty() {
                    return Err(ann.make(ErrStr(format!(
                        "{} uncovered ctors",
                        ctors.len()
                    ))));
                }
                Ok(())
            }
            Compute::CoMatch { arms, ann, .. } => {
                if let TCtor::Var(tvar) = &typ.ctor {
                    let coda = ctx.coda.get(tvar).ok_or_else(|| {
                        ann.make(ErrStr(format!("unknown coda: {}", tvar)))
                    })?;
                    let mut dtors: HashMap<DtorV, (Vec<Type>, Type)> =
                        HashMap::from_iter(coda.dtors.iter().map(
                            |(dtor, ty_args, tret)| {
                                (dtor.clone(), (ty_args.clone(), tret.clone()))
                            },
                        ));
                    for (dtor, vars, body) in arms {
                        let (ty_args, tret) =
                            dtors.remove(dtor).ok_or_else(|| {
                                ann.make(ErrStr(format!(
                                    "unknown dtor: {}",
                                    dtor
                                )))
                            })?;
                        typ.eqv(&tret).ok_or_else(|| {
                            ann.make(TypeMismatch {
                                expected: typ.to_owned(),
                                found: tret,
                            })
                        })?;
                        // check if the dtor has the right number of arguments
                        if vars.len() != ty_args.len() {
                            return Err(ann.make(ArityMismatch {
                                context: format!("`comatch` arm for {}", dtor),
                                expected: vars.len(),
                                found: ty_args.len(),
                            }));
                        }
                        // check the body of the branch
                        let mut ctx = ctx.clone();
                        ctx.extend(
                            vars.iter().cloned().zip(ty_args.iter().cloned()),
                        );
                        body.ana(typ, &ctx)?;
                    }
                    // check that all dtors were covered
                    if !dtors.is_empty() {
                        return Err(ann.make(ErrStr(format!(
                            "{} uncovered dtors",
                            dtors.len()
                        ))));
                    }
                    Ok(())
                } else {
                    Err(ann.make(TypeExpected {
                        expected: format!("a?"),
                        found: typ.to_owned(),
                    }))
                }
            }
            Compute::CoApp { body, dtor, args, ann, .. } => {
                let tscrut = body.syn(ctx)?;
                if let TCtor::Var(tvar) = tscrut.ctor {
                    let coda = ctx.coda.get(&tvar).ok_or_else(|| {
                        ann.make(ErrStr(format!("unknown codata: {}", tvar)))
                    })?;
                    let (_, ty_args, tret) = coda
                        .dtors
                        .iter()
                        .find(|(dtor_, _, _)| dtor == dtor_)
                        .ok_or_else(|| {
                            ann.make(ErrStr(format!("unknown dtor: {}", dtor)))
                        })?;
                    typ.eqv(&tret).ok_or_else(|| {
                        ann.make(TypeMismatch {
                            expected: typ.to_owned(),
                            found: tret.to_owned(),
                        })
                    })?;
                    if args.len() != ty_args.len() {
                        return Err(ann.make(ArityMismatch {
                            context: format!(
                                "application of destructor {}",
                                dtor
                            ),
                            expected: ty_args.len(),
                            found: args.len(),
                        }));
                    }
                    for (arg, expected) in args.iter().zip(ty_args.iter()) {
                        arg.ana(expected, ctx)?;
                    }
                    Ok(())
                } else {
                    Err(ann.make(TypeExpected {
                        expected: format!("a?"),
                        found: tscrut.into(),
                    }))
                }
            }
            c => {
                let t = self.syn(ctx)?;
                t.eqv(typ).ok_or_else(|| {
                    c.ann().make(TypeMismatch {
                        expected: typ.to_owned(),
                        found: t,
                    })
                })
            }
        }
    }
}

impl Type {
    fn internal(name: &'static str, args: Vec<Type>, ann: AnnInfo) -> Self {
        Type {
            ctor: TCtor::Var(TypeV::new(name.to_owned(), ann.clone())),
            args,
            ann,
        }
    }
}

impl TypeCheck for Type {
    type Out = Kind;

    fn syn(&self, ctx: &Ctx) -> Result<Self::Out, Ann<TypeCheckError>> {
        match &self.ctor {
            TCtor::Var(x) => ctx.tmap.get(&x).map_or(
                Err(self.ann.make(TypeCheckError::NameResolve(
                    NameResolveError::UnknownIdentifier {
                        name: x.name().to_owned(),
                        ann: self.ann.clone(),
                    },
                ))),
                |Arity(params, out)| {
                    if self.args.len() != params.len() {
                        Err(self.ann.make(ArityMismatch {
                            context: format!("{}", self),
                            expected: params.len(),
                            found: self.args.len(),
                        }))
                    } else {
                        for (arg, param) in self.args.iter().zip(params) {
                            let karg = arg.syn(ctx)?;
                            param.eqv(&karg).ok_or_else(|| {
                                self.ann.make(KindMismatch {
                                    context: format!(
                                        "synthesizing kind of {}",
                                        x
                                    ),
                                    expected: param.clone(),
                                    found: karg,
                                })
                            })?
                        }
                        Ok(out.clone())
                    }
                },
            ),
            TCtor::OS => {
                if self.args.len() != 0 {
                    Err(self.ann.make(ArityMismatch {
                        context: format!("{}", self),
                        expected: 0,
                        found: self.args.len(),
                    }))
                } else {
                    Ok(Kind::CType)
                }
            }
            TCtor::Ret => {
                if self.args.len() != 1 {
                    Err(self.ann.make(ArityMismatch {
                        context: format!("{}", self),
                        expected: 1,
                        found: self.args.len(),
                    }))
                } else {
                    self.args[0]
                        .syn(ctx)?
                        .ensure_vtype("type argument to Ret", &ann(0, 0))?;
                    Ok(Kind::CType)
                }
            }
            TCtor::Thunk => {
                if self.args.len() != 1 {
                    Err(self.ann.make(ArityMismatch {
                        context: format!("{}", self),
                        expected: 1,
                        found: self.args.len(),
                    }))
                } else {
                    self.args[0].syn(ctx)?.expect_comptype(
                        "type argument to Thunk",
                        &ann(0, 0),
                    )?;
                    Ok(Kind::VType)
                }
            }
            TCtor::Fun => {
                if self.args.len() != 2 {
                    Err(self.ann.make(ArityMismatch {
                        context: format!("{}", self),
                        expected: 1,
                        found: self.args.len(),
                    }))
                } else {
                    self.args[0].syn(ctx)?.ensure_vtype(
                        "domain of a function type",
                        &ann(0, 0),
                    )?;
                    self.args[1].syn(ctx)?.expect_comptype(
                        "codomain of a function type",
                        &ann(0, 0),
                    )?;
                    Ok(Kind::CType)
                }
            }
        }
    }
}

pub trait Eqv {
    fn eqv(&self, other: &Self) -> Option<()>;
}

impl Eqv for () {
    fn eqv(&self, other: &Self) -> Option<()> {
        (self == other).then_some(())
    }
}

impl Eqv for Kind {
    fn eqv(&self, other: &Self) -> Option<()> {
        (self == other).then_some(())
    }
}

impl Eqv for TCtor {
    fn eqv(&self, other: &Self) -> Option<()> {
        match (self, other) {
            (TCtor::Var(x), TCtor::Var(y)) => (x == y).then_some(()),
            (TCtor::OS, TCtor::OS)
            | (TCtor::Ret, TCtor::Ret)
            | (TCtor::Thunk, TCtor::Thunk)
            | (TCtor::Fun, TCtor::Fun) => Some(()),
            (TCtor::Var(..), _)
            | (TCtor::OS, _)
            | (TCtor::Ret, _)
            | (TCtor::Thunk, _)
            | (TCtor::Fun, _) => None,
        }
    }
}

impl Eqv for Type {
    fn eqv(&self, other: &Self) -> Option<()> {
        // Note: being nominal here
        // Note: assumes all type constructors are injective (true for now)
        TCtor::eqv(&self.ctor, &other.ctor)?;
        (self.args.len() == other.args.len()).then_some(())?;
        for (argl, argr) in self.args.iter().zip(&other.args) {
            Type::eqv(argl, &argr)?
        }
        Some(())
    }
}

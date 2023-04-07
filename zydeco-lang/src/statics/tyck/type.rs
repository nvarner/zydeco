use super::*;
use crate::utils::span::span;

impl Type {
    pub fn internal(name: &'static str, args: Vec<RcType>) -> Self {
        TypeApp { tvar: TypeV::new(name.into(), span(0, 0)), args }.into()
    }
    pub fn make_thunk(arg: RcType) -> Self {
        Type::internal("Thunk", vec![arg])
    }
    pub fn elim_thunk(self, ctx: Ctx, span: &SpanInfo) -> Option<Type> {
        let ty = self.lub(Type::make_thunk(rc!(span.make(Hole.into()))), ctx, span).ok()?;
        let SynType::TypeApp(ty_app) = ty.synty else {
            None?
        };
        ty_app.elim_thunk_syntax()
    }
    pub fn make_ret(arg: RcType) -> Self {
        Type::internal("Ret", vec![arg])
    }
    pub fn elim_ret(self, ctx: Ctx, span: &SpanInfo) -> Option<Type> {
        let ty = self.lub(Type::make_ret(rc!(span.make(Hole.into()))), ctx, span).ok()?;
        let SynType::TypeApp(ty_app) = ty.synty else {
            None?
        };
        ty_app.elim_ret_syntax()
    }
    pub fn make_os() -> Self {
        Type::internal("OS", vec![])
    }
    pub fn elim_os(self, ctx: Ctx, span: &SpanInfo) -> Option<()> {
        self.lub(Type::make_os(), ctx, span).map(|_| ()).ok()
    }
}
impl TypeApp<TypeV, RcType> {
    pub fn elim_thunk_syntax(&self) -> Option<Type> {
        if self.tvar.name() == "Thunk" {
            Some(self.args.first().unwrap().inner_ref().clone())
        } else {
            None
        }
    }
    pub fn elim_ret_syntax(&self) -> Option<Type> {
        if self.tvar.name() == "Ret" {
            Some(self.args.first().unwrap().inner_ref().clone())
        } else {
            None
        }
    }
    pub fn elim_os_syntax(&self) -> Option<()> {
        if self.tvar.name() == "OS" {
            Some(())
        } else {
            None
        }
    }
}

impl Ctx {
    pub(super) fn resolve_data(
        &self, ty: Type, span: &SpanInfo,
    ) -> Result<(Data<TypeV, Kind, CtorV, RcType>, Vec<RcType>), TyckError> {
        let SynType::TypeApp(TypeApp { tvar, args }) = ty.synty else {
            Err(self.err(span, TypeExpected {
                context: format!("resolve data"),
                expected: format!("type application"),
                found: ty,
            }))?
        };
        if let Some(Alias { name, params, ty }) = self.alias_env.get(&tvar) {
            let ty = ty.inner_ref().clone();
            let diff = Env::init(&params, &args, || {
                span.make(ArityMismatch {
                    context: format!("data type `{}` instiantiation", name),
                    expected: params.len(),
                    found: args.len(),
                })
            })
            .map_err(|e| e.traced(self.trace.clone()))?;
            self.resolve_data(ty.subst(diff).map_err(|e| e.traced(self.trace.clone()))?, span)
        } else {
            let data = self.data_env.get(&tvar).cloned().ok_or_else(|| {
                self.err(span, NameResolveError::UnboundTypeVariable { tvar }.into())
            })?;
            Ok((data, args))
        }
    }
    pub(super) fn resolve_codata(
        &self, ty: Type, span: &SpanInfo,
    ) -> Result<(Codata<TypeV, Kind, DtorV, RcType>, Vec<RcType>), TyckError> {
        let SynType::TypeApp(TypeApp { tvar, args }) = ty.synty else {
            Err(self.err(span, TypeExpected {
                context: format!("resolve codata"),
                expected: format!("type application"),
                found: ty,
            }))?
        };
        if let Some(Alias { name, params, ty }) = self.alias_env.get(&tvar) {
            let ty = ty.inner_ref().clone();
            let diff = Env::init(&params, &args, || {
                span.make(ArityMismatch {
                    context: format!("data type `{}` instiantiation", name),
                    expected: params.len(),
                    found: args.len(),
                })
            })
            .map_err(|e| e.traced(self.trace.clone()))?;
            self.resolve_codata(ty.subst(diff).map_err(|e| e.traced(self.trace.clone()))?, span)
        } else {
            let codata = self.codata_env.get(&tvar).cloned().ok_or_else(|| {
                self.err(span, NameResolveError::UnboundTypeVariable { tvar }.into())
            })?;
            Ok((codata, args))
        }
    }
}

impl TypeCheck for Span<Type> {
    type Ctx = Ctx;
    type Out = Kind;
    fn syn_step(
        &self, mut ctx: Self::Ctx,
    ) -> Result<Step<(Self::Ctx, &Self), Self::Out>, TyckError> {
        ctx.trace.push(Frame {
            tycker_src: format!("{}:{}:{}", file!(), line!(), column!()),
            sort: "syn type".to_owned(),
            term: format!("{}", self.inner_ref().fmt()),
            info: self.span().clone(),
        });
        let span = self.span();
        let ty = self
            .inner_ref()
            .clone()
            .subst(ctx.type_env.clone())
            .map_err(|e| e.traced(ctx.trace.clone()))?;
        match &ty.synty {
            SynType::TypeApp(app) => {
                let tvar = &app.tvar;
                // type constructor
                let Some(TypeArity { params, kd }) = ctx.type_ctx.get(&tvar) else {
                    Err(ctx.err(span,
                        NameResolveError::UnboundTypeVariable {
                            tvar: tvar.to_owned(),
                        }.into()
                    ))?
                };
                bool_test(app.args.len() == params.len(), || {
                    ctx.err(
                        span,
                        ArityMismatch {
                            context: format!("{}", self.inner_ref().fmt()),
                            expected: params.len(),
                            found: app.args.len(),
                        },
                    )
                })?;
                for (arg, kd) in app.args.iter().zip(params.iter()) {
                    arg.ana(kd.clone(), ctx.clone())?;
                }
                Ok(Step::Done(kd.clone()))
            }
            SynType::Forall(Forall { param: (param, kd), ty }) => {
                ctx.type_ctx.insert(param.clone(), TypeArity { params: vec![], kd: kd.clone() });
                ty.ana(Kind::CType, ctx)?;
                Ok(Step::Done(Kind::CType))
            }
            SynType::Exists(Exists { param: (param, kd), ty }) => {
                ctx.type_ctx.insert(param.clone(), TypeArity { params: vec![], kd: kd.clone() });
                ty.ana(Kind::VType, ctx)?;
                Ok(Step::Done(Kind::VType))
            }
            SynType::AbstVar(AbstVar(abs)) => Ok(Step::Done(ctx.abst_ctx[*abs])),
            SynType::Hole(_) => Err(ctx.err(span, NeedAnnotation { content: format!("hole") }))?,
        }
    }
    fn ana_step(
        &self, kd: Self::Out, mut ctx: Self::Ctx,
    ) -> Result<Step<(Self::Ctx, &Self), Self::Out>, TyckError> {
        ctx.trace.push(Frame {
            tycker_src: format!("{}:{}:{}", file!(), line!(), column!()),
            sort: format!("ana type with kind {}", kd.fmt()),
            term: format!("{}", self.inner_ref().fmt()),
            info: self.span().clone(),
        });
        let span = self.span();
        let ty = self
            .inner_ref()
            .clone()
            .subst(ctx.type_env.clone())
            .map_err(|e| e.traced(ctx.trace.clone()))?;
        match ty.synty {
            SynType::Hole(_) => Ok(Step::Done(kd)),
            SynType::TypeApp(_) | SynType::Forall(_) | SynType::Exists(_) | SynType::AbstVar(_) => {
                let kd_syn = self.syn(ctx.clone())?;
                kd_syn.lub(kd, Default::default(), span)?;
                Ok(Step::Done(kd))
            }
        }
    }
}

impl Type {
    pub(super) fn subst(self, mut diff: Env<TypeV, Type>) -> Result<Self, Span<TyckErrorItem>> {
        match self.synty {
            SynType::TypeApp(TypeApp { tvar, mut args }) => {
                if let Some(ty) = diff.get(&tvar) {
                    bool_test(args.is_empty(), || {
                        tvar.span().make(ArityMismatch {
                            context: format!("type variable `{}`", tvar),
                            expected: 0,
                            found: args.len(),
                        })
                    })?;
                    Ok(ty.clone())
                } else {
                    for arg in args.iter_mut() {
                        *arg = rc!(arg.as_ref().clone().try_map(|ty| ty.subst(diff.clone()))?);
                    }
                    Ok(Type { synty: TypeApp { tvar, args }.into() })
                }
            }
            SynType::Forall(Forall { param, ty }) => {
                diff.remove(&param.0);
                Ok(Type {
                    synty: Forall {
                        param,
                        ty: rc!(ty.as_ref().clone().try_map(|ty| ty.subst(diff.clone()))?),
                    }
                    .into(),
                })
            }
            SynType::Exists(Exists { param, ty }) => {
                diff.remove(&param.0);
                Ok(Type {
                    synty: Exists {
                        param,
                        ty: rc!(ty.as_ref().clone().try_map(|ty| ty.subst(diff.clone()))?),
                    }
                    .into(),
                })
            }
            SynType::AbstVar(_) | SynType::Hole(_) => Ok(self),
        }
    }
    // pub(super) fn lub(
    //     lhs: Self, rhs: Self, ctx: Ctx, f: impl FnOnce() -> TyckError + Clone,
    // ) -> Result<Self, TyckError> {
    //     let lhs = lhs.subst(ctx.type_env.clone()).map_err(|e| e.traced(ctx.trace.clone()))?;
    //     let rhs = rhs.subst(ctx.type_env.clone()).map_err(|e| e.traced(ctx.trace.clone()))?;
    //     match (&lhs.synty, &rhs.synty) {
    //         (SynType::Hole(_), _) => Ok(rhs),
    //         (_, SynType::Hole(_)) => Ok(lhs),
    //         (SynType::TypeApp(lhs), SynType::TypeApp(rhs)) => {
    //             bool_test(lhs.tvar == rhs.tvar, f.clone())?;
    //             let mut args = vec![];
    //             for (lhs, rhs) in (lhs.args.iter()).zip(rhs.args.iter()) {
    //                 let arg = Self::lub(
    //                     lhs.inner_ref().clone(),
    //                     rhs.inner_ref().clone(),
    //                     ctx.clone(),
    //                     f.clone(),
    //                 )?;
    //                 args.push(rc!(lhs.span().make(arg)));
    //             }
    //             Ok(TypeApp { tvar: lhs.tvar.clone(), args }.into())
    //         }
    //         (SynType::Forall(lhs), SynType::Forall(rhs)) => {
    //             bool_test(lhs.param == rhs.param, f.clone())?;
    //             let ty = Self::lub(lhs.ty.inner_ref().clone(), rhs.ty.inner_ref().clone(), ctx, f)?;
    //             Ok(Forall { param: lhs.param.clone(), ty: rc!(lhs.ty.span().make(ty)) }.into())
    //         }
    //         (SynType::Exists(lhs), SynType::Exists(rhs)) => {
    //             bool_test(lhs.param == rhs.param, f.clone())?;
    //             let ty = Self::lub(lhs.ty.inner_ref().clone(), rhs.ty.inner_ref().clone(), ctx, f)?;
    //             Ok(Exists { param: lhs.param.clone(), ty: rc!(lhs.ty.span().make(ty)) }.into())
    //         }
    //         (SynType::AbstVar(lhs), SynType::AbstVar(rhs)) => {
    //             bool_test(lhs == rhs, f)?;
    //             Ok(lhs.clone().into())
    //         }
    //         (SynType::TypeApp(_), _)
    //         | (SynType::Forall(_), _)
    //         | (SynType::Exists(_), _)
    //         | (SynType::AbstVar(_), _) => Err(f()),
    //     }
    // }
}

impl Monoid for Env<TypeV, Type> {
    fn empty() -> Self {
        Self::new()
    }

    fn append(self, ori: Self) -> Self {
        // append on Env is actually composing lazy substitutions, effectively
        //       M [\gamma] [\delta] = M [\delta . \gamma]
        // where we refer to gamma as "original" and delta as "diff" then
        //      new = append(diff, original)
        let mut new = Self::new();
        for (x, ty) in self.clone() {
            if !ori.contains_key(&x) {
                new.insert(x, ty);
            }
        }
        for (x, ty) in ori {
            new.insert(x, ty.subst(self.clone()).unwrap());
        }
        new
    }
}

impl Env<TypeV, Type> {
    pub(super) fn init(
        params: &[(TypeV, Kind)], ty_app_args: &[RcType],
        arity_err: impl FnOnce() -> Span<TyckErrorItem>,
    ) -> Result<Self, Span<TyckErrorItem>> {
        bool_test(params.len() == ty_app_args.len(), arity_err)?;
        Ok(Env::from_iter(
            params
                .iter()
                .map(|(tvar, _)| tvar.to_owned())
                .zip(ty_app_args.iter().map(|arg| arg.inner_ref().to_owned())),
        ))
    }
}
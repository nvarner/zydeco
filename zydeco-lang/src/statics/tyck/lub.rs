use super::*;

impl Lub for () {
    type Ctx = Ctx;
    type Out = ();
    fn lub(self, _rhs: (), _ctx: Ctx, _span: &SpanInfo) -> Result<(), TyckError> {
        Ok(())
    }
}

impl<T> Lub for Seal<T> {
    type Ctx = ();
    type Out = Seal<T>;
    fn lub(self, _other: Self, _: Self::Ctx, _: &SpanInfo) -> Result<Self::Out, TyckError> {
        unreachable!()
    }
}

impl Lub for Kind {
    type Ctx = Ctx;
    type Out = Kind;
    fn lub(self, rhs: Kind, ctx: Ctx, span: &SpanInfo) -> Result<Kind, TyckError> {
        bool_test(self == rhs, || {
            ctx.err(span, KindMismatch { context: format!("lub"), expected: self, found: rhs })
        })?;
        Ok(self)
    }
}

impl Lub for Type {
    type Ctx = Ctx;
    type Out = Type;
    fn lub(self, rhs: Type, mut ctx: Ctx, span: &SpanInfo) -> Result<Type, TyckError> {
        let lhs = self;
        let err = {
            let expected = lhs.clone();
            let found = rhs.clone();
            || ctx.err(span, TypeMismatch { context: format!("lub"), expected, found })
        };
        // let lhs = self.subst(ctx.type_env.clone(), ctx.clone())?;
        // let rhs = rhs.subst(ctx.type_env.clone(), ctx.clone())?;
        match (&lhs.synty, &rhs.synty) {
            // (SynType::Hole(_), SynType::Hole(_)) => Err(err())?,
            (SynType::Hole(_), _) => Ok(rhs),
            (_, SynType::Hole(_)) => Ok(lhs),
            (SynType::TypeApp(lhs), _) if ctx.type_env.contains_key(&lhs.tvar) => {
                // lhs is a type variable
                let ty = ctx.clone().type_env[&lhs.tvar].clone();
                ty.lub(rhs, ctx, span)
            }
            (_, SynType::TypeApp(rhs)) if ctx.type_env.contains_key(&rhs.tvar) => {
                // rhs is a type variable
                let ty = ctx.clone().type_env[&rhs.tvar].clone();
                lhs.lub(ty, ctx, span)
            }
            (SynType::TypeApp(lhs), _) if ctx.alias_env.contains_key(&lhs.tvar) => {
                // lhs is a type variable
                let Alias { name, params, ty } = ctx.clone().alias_env[&lhs.tvar].clone();
                let diff = Env::init(&params, &lhs.args, || {
                    ctx.err(
                        span,
                        ArityMismatch {
                            context: format!("data type `{}` instiantiation", name),
                            expected: params.len(),
                            found: lhs.args.len(),
                        },
                    )
                })?;
                let ty = ty.inner_ref().clone().subst(diff, ctx.clone())?;
                ty.lub(rhs, ctx, span)
            }
            (_, SynType::TypeApp(rhs)) if ctx.alias_env.contains_key(&rhs.tvar) => {
                // lhs is a type variable
                let Alias { name, params, ty } = ctx.clone().alias_env[&rhs.tvar].clone();
                let diff = Env::init(&params, &rhs.args, || {
                    ctx.err(
                        span,
                        ArityMismatch {
                            context: format!("data type `{}` instiantiation", name),
                            expected: params.len(),
                            found: rhs.args.len(),
                        },
                    )
                })?;
                let ty = ty.inner_ref().clone().subst(diff, ctx.clone())?;
                lhs.lub(ty, ctx, span)
            }
            (SynType::TypeApp(lhs), SynType::TypeApp(rhs)) => {
                bool_test(lhs.tvar == rhs.tvar, err)?;
                let mut args = vec![];
                for (lhs, rhs) in (lhs.args.iter()).zip(rhs.args.iter()) {
                    let arg = Self::lub(
                        lhs.inner_ref().clone(),
                        rhs.inner_ref().clone(),
                        ctx.clone(),
                        span,
                    )?;
                    args.push(rc!(lhs.span().make(arg)));
                }
                Ok(TypeApp { tvar: lhs.tvar.clone(), args }.into())
            }
            (
                SynType::Forall(Forall { param: (tvar, kd), ty }),
                SynType::Forall(Forall { param: (tvar_, kd_), ty: ty_ }),
            ) => {
                bool_test(kd == kd_, err)?;
                let abst_var = ctx.fresh(kd.clone());
                let lhs_ty = (ty.inner_ref().clone()).subst(
                    Env::from_iter([(tvar.clone(), abst_var.clone().into())]),
                    ctx.clone(),
                )?;
                let rhs_ty = (ty_.inner_ref().clone())
                    .subst(Env::from_iter([(tvar_.clone(), abst_var.into())]), ctx.clone())?;
                let _ty = lhs_ty.lub(rhs_ty, ctx, span)?;
                // HACK: needs revertable type subst
                // Ok(Forall { param: lhs.param.clone(), ty: rc!(lhs.ty.span().make(ty)) }.into())
                Ok(lhs.clone())
            }
            (
                SynType::Exists(Exists { param: (tvar, kd), ty }),
                SynType::Exists(Exists { param: (tvar_, kd_), ty: ty_ }),
            ) => {
                bool_test(kd == kd_, err)?;
                let abst_var = ctx.fresh(kd.clone());
                let lhs_ty = (ty.inner_ref().clone()).subst(
                    Env::from_iter([(tvar.clone(), abst_var.clone().into())]),
                    ctx.clone(),
                )?;
                let rhs_ty = (ty_.inner_ref().clone())
                    .subst(Env::from_iter([(tvar_.clone(), abst_var.into())]), ctx.clone())?;
                let _ty = lhs_ty.lub(rhs_ty, ctx, span)?;
                // HACK: needs revertable type subst
                // Ok(Exists { param: lhs.param.clone(), ty: rc!(lhs.ty.span().make(ty)) }.into())
                Ok(lhs.clone())
            }
            (SynType::AbstVar(lhs), SynType::AbstVar(rhs)) => {
                bool_test(lhs == rhs, err)?;
                Ok(lhs.clone().into())
            }
            (SynType::TypeApp(_), _)
            | (SynType::Forall(_), _)
            | (SynType::Exists(_), _)
            | (SynType::AbstVar(_), _) => Err(err()),
        }
    }
}

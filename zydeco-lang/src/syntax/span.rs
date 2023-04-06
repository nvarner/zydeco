use super::*;
use crate::utils::span::*;

impl SpanHolder for Hole {
    fn span_map_mut<F>(&mut self, _f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
    }
}

impl SpanHolder for Kind {
    fn span_map_mut<F>(&mut self, _f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
    }
}

impl<K> SpanHolder for TypeArity<K>
where
    K: KindT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let TypeArity { params, kd } = self;
        for param in params {
            param.span_map_mut(f.clone());
        }
        kd.span_map_mut(f);
    }
}

impl<TyV, T> SpanHolder for TypeApp<TyV, T>
where
    TyV: TyVarT + SpanHolder,
    T: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let TypeApp { tvar, args } = self;
        tvar.span_map_mut(f.clone());
        for arg in args {
            arg.span_map_mut(f.clone());
        }
    }
}

impl<TyV, Ty> SpanHolder for Forall<TyV, Ty>
where
    TyV: TyVarT + SpanHolder,
    Ty: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Forall { param, ty } = self;
        param.span_map_mut(f.clone());
        ty.span_map_mut(f);
    }
}

impl<TyV, Ty> SpanHolder for Exists<TyV, Ty>
where
    TyV: TyVarT + SpanHolder,
    Ty: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Exists { param, ty } = self;
        param.span_map_mut(f.clone());
        ty.span_map_mut(f);
    }
}

impl<Term, Type> SpanHolder for Annotation<Term, Type>
where
    Term: SpanHolder,
    Type: SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Annotation { term, ty } = self;
        term.span_map_mut(f.clone());
        ty.span_map_mut(f);
    }
}

impl<B> SpanHolder for Thunk<B>
where
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Thunk(body) = self;
        body.span_map_mut(f);
    }
}

impl SpanHolder for Literal {
    fn span_map_mut<F>(&mut self, _f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
    }
}

impl<C, A> SpanHolder for Ctor<C, A>
where
    C: CtorT + SpanHolder,
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Ctor { ctor, args } = self;
        ctor.span_map_mut(f.clone());
        for arg in args {
            arg.span_map_mut(f.clone());
        }
    }
}

impl<Ty, A> SpanHolder for Pack<Ty, A>
where
    Ty: TypeT + SpanHolder,
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Pack { ty, body: val } = self;
        ty.span_map_mut(f.clone());
        val.span_map_mut(f);
    }
}

impl<A> SpanHolder for Ret<A>
where
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Ret(arg) = self;
        arg.span_map_mut(f);
    }
}

impl<A> SpanHolder for Force<A>
where
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Force(arg) = self;
        arg.span_map_mut(f);
    }
}

impl<TeV, A, B> SpanHolder for Let<TeV, A, B>
where
    TeV: VarT + SpanHolder,
    A: ValueT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Let { var, def, body } = self;
        var.span_map_mut(f.clone());
        def.span_map_mut(f.clone());
        body.span_map_mut(f);
    }
}

impl<TeV, B1, B2> SpanHolder for Do<TeV, B1, B2>
where
    TeV: VarT + SpanHolder,
    B1: ComputationT + SpanHolder,
    B2: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Do { var, comp, body } = self;
        var.span_map_mut(f.clone());
        comp.span_map_mut(f.clone());
        body.span_map_mut(f);
    }
}

impl<TeV, B> SpanHolder for Rec<TeV, B>
where
    TeV: VarT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Rec { var, body } = self;
        var.span_map_mut(f.clone());
        body.span_map_mut(f);
    }
}

impl<C, TeV, A, B> SpanHolder for Match<C, TeV, A, B>
where
    C: CtorT + SpanHolder,
    TeV: VarT + SpanHolder,
    A: ValueT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Match { scrut, arms } = self;
        scrut.span_map_mut(f.clone());
        for Matcher { ctor, vars, body } in arms {
            ctor.span_map_mut(f.clone());
            for var in vars {
                var.span_map_mut(f.clone());
            }
            body.span_map_mut(f.clone());
        }
    }
}

impl<D, TeV, B> SpanHolder for CoMatch<D, TeV, B>
where
    D: DtorT + SpanHolder,
    TeV: VarT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let CoMatch { arms } = self;
        for CoMatcher { dtor, vars, body } in arms {
            dtor.span_map_mut(f.clone());
            for var in vars {
                var.span_map_mut(f.clone());
            }
            body.span_map_mut(f.clone());
        }
    }
}

impl<B, D, A> SpanHolder for Dtor<B, D, A>
where
    B: ComputationT + SpanHolder,
    D: DtorT + SpanHolder,
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Dtor { body, dtor, args } = self;
        body.span_map_mut(f.clone());
        dtor.span_map_mut(f.clone());
        for arg in args {
            arg.span_map_mut(f.clone());
        }
    }
}

impl<TyV, Kd, B> SpanHolder for TypAbs<TyV, Kd, B>
where
    TyV: TyVarT + SpanHolder,
    Kd: KindT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let TypAbs { tvar, kd, body } = self;
        tvar.span_map_mut(f.clone());
        kd.span_map_mut(f.clone());
        body.span_map_mut(f);
    }
}

impl<B, Ty> SpanHolder for TypApp<B, Ty>
where
    B: ComputationT + SpanHolder,
    Ty: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let TypApp { body, arg } = self;
        body.span_map_mut(f.clone());
        arg.span_map_mut(f);
    }
}

impl<A, TyV, TeV, B> SpanHolder for MatchPack<A, TyV, TeV, B>
where
    A: ValueT + SpanHolder,
    TyV: TyVarT + SpanHolder,
    TeV: VarT + SpanHolder,
    B: ComputationT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let MatchPack { scrut, tvar, var, body } = self;
        scrut.span_map_mut(f.clone());
        tvar.span_map_mut(f.clone());
        var.span_map_mut(f.clone());
        body.span_map_mut(f);
    }
}

impl<T: SpanHolder> SpanHolder for DeclSymbol<T> {
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        self.inner.span_map_mut(f);
    }
}

impl<TyV, C, T> SpanHolder for Data<TyV, C, T>
where
    TyV: TyVarT + SpanHolder,
    C: CtorT + SpanHolder,
    T: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Data { name, params, ctors } = self;
        name.span_map_mut(f.clone());
        for (ty, kd) in params {
            ty.span_map_mut(f.clone());
            kd.span_map_mut(f.clone());
        }
        for DataBr(ctor, tys) in ctors {
            ctor.span_map_mut(f.clone());
            for ty in tys {
                ty.span_map_mut(f.clone());
            }
        }
    }
}

impl<TyV, D, T> SpanHolder for Codata<TyV, D, T>
where
    TyV: TyVarT + SpanHolder,
    D: DtorT + SpanHolder,
    T: TypeT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Codata { name, params, dtors } = self;
        name.span_map_mut(f.clone());
        for (ty, kd) in params {
            ty.span_map_mut(f.clone());
            kd.span_map_mut(f.clone());
        }
        for CodataBr(dtor, tys, ty) in dtors {
            dtor.span_map_mut(f.clone());
            for ty in tys {
                ty.span_map_mut(f.clone());
            }
            ty.span_map_mut(f.clone());
        }
    }
}

impl<TeV, A> SpanHolder for Define<TeV, A>
where
    TeV: VarT + SpanHolder,
    A: ValueT + SpanHolder,
{
    fn span_map_mut<F>(&mut self, f: F)
    where
        F: Fn(&mut SpanInfo) + Clone,
    {
        let Define { name, def } = self;
        name.span_map_mut(f.clone());
        def.span_map_mut(f);
    }
}

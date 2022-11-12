use std::fmt::Debug;

pub trait AnnT: Clone + Debug {
    type Span;
    fn internal(sort: &'static str) -> Self;
    fn span(&self) -> Self::Span;
}

impl AnnT for () {
    type Span = ();
    fn internal(_: &'static str) -> Self {}
    fn span(&self) -> Self::Span {}
}

pub trait AnnHolder<Ann: AnnT> {
    fn ann(&self) -> &Ann;
    fn set_ann(&mut self, ann: Ann);
}

pub trait AnnMap<Ann: AnnT, Ann2: AnnT> {
    type Output;
    fn map_ann<F: Fn(&Ann) -> Ann2>(&self, f: F) -> Self::Output;
}

impl<Ann: AnnT, T: AnnHolder<Ann> + Clone> AnnMap<Ann, Ann> for T {
    type Output = Self;
    fn map_ann<F: Fn(&Ann) -> Ann>(&self, f: F) -> Self::Output {
        let mut t = self.clone();
        t.set_ann(f(self.ann()));
        t
    }
}
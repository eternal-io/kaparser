use super::*;

#[inline(always)]
pub const fn map<'i, U, P, F, Ḟ>(f: F, pat: P) -> Map<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Ḟ,
    Ḟ: 'static,
{
    Map {
        pat,
        map: f,
        phantom: PhantomData,
    }
}

#[inline(always)]
pub const fn filter_map<'i, U, P, F, Ḟ>(f: F, pat: P) -> FilterMap<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Option<Ḟ>,
    Ḟ: 'static,
{
    FilterMap {
        pat,
        filter_map: f,
        phantom: PhantomData,
    }
}

//------------------------------------------------------------------------------

pub struct Map<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Ḟ,
{
    pat: P,
    map: F,
    phantom: PhantomData<(&'i U, Ḟ)>,
}

impl<'i, U, P, F, Ḟ> Pattern<'i, U> for Map<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Ḟ,
{
    type Captured = Ḟ;
    type Internal = P::Internal;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        self.pat.init()
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        self.pat.precede(slice, entry, eof)
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        (self.map)(self.pat.extract(slice, entry))
    }
}

//------------------------------------------------------------------------------

pub struct FilterMap<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Option<Ḟ>,
    Ḟ: 'static,
{
    pat: P,
    filter_map: F,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, F, Ḟ> Pattern<'i, U> for FilterMap<'i, U, P, F, Ḟ>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U>,
    F: Fn(P::Captured) -> Option<Ḟ>,
    Ḟ: 'static,
{
    type Captured = P::Captured;
    type Internal = InnerAlt2<P::Internal, Ḟ>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        InnerAlt2::Var1(self.pat.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        todo!()
        // let (t, len) = self.pat.precede(slice, entry, eof)?;
        // match t {
        //     Transfer::Accepted => {
        //         todo!();
        //     }
        //     t => Ok((t, len)),
        // }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        todo!()
    }
}

//------------------------------------------------------------------------------

pub struct FilterStatic<'i, U, P, Ṗ, F>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U, Captured = Ṗ>,
    Ṗ: 'static,
    F: Fn(&Ṗ) -> bool,
{
    pat: P,
    filter: F,
    phantom: PhantomData<&'i U>,
}

impl<'i, U, P, Ṗ, F> Pattern<'i, U> for FilterStatic<'i, U, P, Ṗ, F>
where
    U: 'i + ?Sized + Slice,
    P: Pattern<'i, U, Captured = Ṗ>,
    Ṗ: 'static,
    F: Fn(&Ṗ) -> bool,
{
    type Captured = P::Captured;
    type Internal = InnerAlt2<P::Internal, Option<P::Captured>>;

    #[inline(always)]
    fn init(&self) -> Self::Internal {
        InnerAlt2::Var1(self.pat.init())
    }
    #[inline(always)]
    fn precede(&self, slice: &'i U, entry: &mut Self::Internal, eof: bool) -> PrecedeResult {
        let InnerAlt2::Var1(state) = entry else {
            panic!("contract violation")
        };

        let (t, len) = self.pat.precede(slice, state, eof)?;
        match t {
            Transfer::Accepted => {
                let InnerAlt2::Var1(state) = mem::replace(entry, InnerAlt2::Var2(None)) else {
                    unreachable!()
                };
                *entry = InnerAlt2::Var2(Some(self.pat.extract(slice, state)));

                let InnerAlt2::Var2(Some(cap)) = entry else {
                    unreachable!()
                };
                match (self.filter)(cap) {
                    true => Ok((Transfer::Accepted, len)),
                    false => Ok((Transfer::Rejected, len)),
                }
            }
            t => Ok((t, len)),
        }
    }
    #[inline(always)]
    fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
        let _ = slice;
        let InnerAlt2::Var2(Some(cap)) = entry else {
            panic!("contract violation")
        };
        cap
    }
}

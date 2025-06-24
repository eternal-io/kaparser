use crate::{input::*, marker};

pub trait Quattrn<'src, U>
where
    U: Input<'src>,
{
    type View<'tmp>
    where
        'src: 'tmp;

    fn fullmatch_v<'tmp>(&self, input: &'tmp mut U) -> Self::View<'tmp>
    where
        'src: 'tmp;

    fn maq<F, Out>(&self, input: &mut U, f: F) -> Out
    where
        F: for<'all> Fn(Self::View<'all>) -> Out,
    {
        f(self.fullmatch_v(input))
    }
}

pub trait Pattern<'src, U>
where
    U: Input<'src>,
{
    type Captured;

    fn fullmatch(&self, input: &mut U) -> Self::Captured;
}

impl<'src, U, Q> Pattern<'src, U> for Q
where
    U::_Marker: marker::Static,
    U: Input<'src>,
    Q: Quattrn<'src, U>,
{
    type Captured = Q::View<'src>;

    fn fullmatch(&self, input: &mut U) -> Self::Captured {
        // SAFETY:
        // This balnket implementation only works for inputs that marked as `StaticInput`,
        // which ensures `'tmp` outlives `'src`, therefore the lifetime can be safely extended.
        // In other words, they are inputs that do not need to be mutated when getting a slice or item.
        unsafe {
            core::mem::transmute(self.fullmatch_v(input))
            // Src = for<'tmp> Q::View<'tmp>;
            // Dst = Q::View<'src>;
        }
    }
}

//------------------------------------------------------------------------------

impl<'src, U> Quattrn<'src, U> for ()
where
    U: InputSlice<'src, Slice = str>,
{
    type View<'tmp>
        = &'tmp str
    where
        'src: 'tmp;

    fn fullmatch_v<'tmp>(&self, input: &'tmp mut U) -> Self::View<'tmp>
    where
        'src: 'tmp,
    {
        let cursor = input.begin();
        input.discard_slice(cursor, 0).0
    }
}

// use alloc::string::String;
// pub fn pat_string<'src>() -> impl Pattern<'src, String, Captured = &'src str> {
//     () // OK, doesn't compile.
// }

// pub fn pat_str<'src>() -> impl Pattern<'src, &'src str, Captured = &'src str> {
//     ()
// }

// #[test]
// fn test_str() {
//     let mut msg = "";
//     let pat = pat_str();
//     let a = pat.fullmatch(&mut msg);
//     let b = pat.fullmatch(&mut msg);
//     println!("{} {}", a, b)
// }

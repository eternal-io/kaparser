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
    U::_Mark: marker::Static,
    U: Input<'src>,
    Q: Quattrn<'src, U>,
{
    type Captured = Q::View<'src>;

    fn fullmatch(&self, input: &mut U) -> Self::Captured {
        unsafe { core::mem::transmute(self.fullmatch_v(input)) }
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
        let mut cursor = input.begin();
        input.discard_slice(&mut cursor, 0)
    }
}

// use alloc::string::String;
// pub fn pat_string<'src>() -> impl Pattern<'src, String, Captured = &'src str> {
//     () // OK, doesn't compile.
// }

pub fn pat_str<'src>() -> impl Pattern<'src, &'src str, Captured = &'src str> {
    ()
}

#[test]
fn test_str() {
    let mut msg = "";
    let pat = pat_str();
    let a = pat.fullmatch(&mut msg);
    let b = pat.fullmatch(&mut msg);
    println!("{} {}", a, b)
}

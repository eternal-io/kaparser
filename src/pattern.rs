use crate::input::*;

pub trait Quattrn<'src, U: Input<'src>> {
    type View<'tmp>;

    fn fullmatch_v<'tmp>(&self, input: &mut U) -> Self::View<'tmp>;
}

pub trait Pattern<'src, U: Input<'src>> {
    type Captured;

    fn fullmatch(&self, input: &mut U) -> Self::Captured;
}

impl<'src, U, Q> Pattern<'src, U> for Q
where
    U: Input<'src>,
    Q: Quattrn<'src, U>,
    for<'tmp> Q::View<'tmp>: 'src,
{
    type Captured = Q::View<'src>;

    fn fullmatch(&self, input: &mut U) -> Self::Captured {
        self.fullmatch_v(input)
    }
}

//------------------------------------------------------------------------------

impl<'src, U> Quattrn<'src, U> for ()
where
    U: for<'tmp> InputSlice<'src, Slice<'tmp> = &'src str>,
{
    type View<'tmp> = U::Slice<'tmp>;

    fn fullmatch_v<'tmp>(&self, input: &mut U) -> Self::View<'tmp> {
        let mut cursor = input.begin();
        input.discard_slice(&mut cursor, 0)
    }
}

use alloc::string::String;

// pub fn q_string<'src>() -> impl for<'tmp> Quattrn<'src, String, View<'tmp> = &'tmp str> {
//     ()
// }

// pub fn p_string<'src>() -> impl Pattern<'src, String, &'src str> {
//     ()
// }

// pub fn q_str<'src>() -> impl for<'tmp> Quattrn<'src, &'src str, View<'tmp> = &'tmp str> {
//     ()
// }

// pub fn p_str<'src>() -> impl Pattern<'src, &'src str, &'src str> {
//     ()
// }

// #[test]
// fn test_string() {
//     let mut msging = String::new();
//     let pat = q_string();
//     let a = pat.fullmatch_v(&mut msging);
//     let b = pat.fullmatch_v(&mut msging);
//     println!("{} {}", a, b)
// }

// #[test]
// fn test_str() {
//     let mut msg = "";
//     let pat = p_str();
//     let a = pat.fullmatch(&mut msg);
//     let b = pat.fullmatch(&mut msg);
//     println!("{} {}", a, b)
// }

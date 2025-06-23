use crate::input::*;

pub trait Quattrn<'src, 'tmp, U>
where
    U: Input<'src, 'tmp>,
{
    type View<'once>
    where
        'src: 'once,
        'once: 'tmp;

    unsafe fn fullmatch_v<'once>(&self, input: &mut U) -> Self::View<'once>
    where
        'src: 'once,
        'once: 'tmp;
}

pub trait Pattern<'src, U>
where
    U: Input<'src, 'src>,
{
    type Captured;

    fn fullmatch(&self, input: &mut U) -> Self::Captured;
}

// impl<'src, U, Q> Pattern<'src, U> for Q
// where
//     U: Input<'src, 'src>,
//     Q: Quattrn<'src, 'src, U>,
//     for<'once> Q::View<'once>: 'src,
// {
//     type Captured = Q::View<'src>;

//     fn fullmatch(&self, input: &mut U) -> Self::Captured {
//         unsafe { self.fullmatch_v(input) }
//     }
// }

//------------------------------------------------------------------------------

impl<'src, 'tmp, U> Quattrn<'src, 'tmp, U> for ()
where
    U: InputSlice<'src, 'tmp, Slice = str>,
{
    type View<'once>
        = &'once U::Slice
    where
        'src: 'once,
        'once: 'tmp;

    unsafe fn fullmatch_v<'once>(&self, input: &mut U) -> Self::View<'once>
    where
        'src: 'once,
        'once: 'tmp,
    {
        let mut cursor = input.begin();
        unsafe { input.discard_slice(&mut cursor, 0) }
    }
}

use alloc::string::String;

// pub fn q_string<'src: 'static, 'tmp>() -> impl for<'once> Quattrn<'src, 'tmp, String, View<'once> = &'once str> {
//     ()
// }

// pub fn p_string<'src>() -> impl Pattern<'src, String, &'src str> {
//     ()
// }

// pub fn q_str<'src: 'static>() -> impl for<'once> Quattrn<'src, 'src, &'src str, View<'once> = &'once str> {
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

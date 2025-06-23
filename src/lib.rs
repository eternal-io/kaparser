#![no_std]

#[cfg(test)]
#[cfg_attr(test, macro_use)]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod common;
pub mod input;
pub mod pattern;
pub mod predicate;
pub mod slice;

//------------------------------------------------------------------------------

pub trait Slice<'src> {
    fn identity(&self) -> &Self {
        self
    }
}

impl<'src> Slice<'src> for &'src str {}

//------------------------------------------------------------------------------

pub trait InputSliceTy<'src, 'tmp, _Alive = &'tmp &'src ()> {
    type Sliced: ?Sized + Slice<'tmp>;

    fn get_slice(&'src mut self) -> Self::Sliced;
}

pub trait InputSlice<'src>: for<'tmp> InputSliceTy<'src, 'tmp> {}

impl<'src, I> InputSlice<'src> for I where I: for<'tmp> InputSliceTy<'src, 'tmp> {}

use alloc::string::String;

impl<'src, 'tmp> InputSliceTy<'src, 'tmp> for &'src str {
    type Sliced = &'tmp str;

    fn get_slice(&'src mut self) -> Self::Sliced {
        *self
    }
}

impl<'src, 'tmp> InputSliceTy<'src, 'tmp> for String {
    type Sliced = &'tmp str;

    fn get_slice(&'src mut self) -> Self::Sliced {
        self.as_str()
    }
}

//------------------------------------------------------------------------------

pub trait QuattrnTy<'src, 'tmp, I, _Alive = &'tmp &'src ()> {
    type View;

    fn fullmatch_v(&self, input: &'src mut I) -> Self::View;
}

impl<'src, 'tmp, I> QuattrnTy<'src, 'tmp, I> for ()
where
    I: InputSliceTy<'src, 'tmp, Sliced = &'tmp str>,
    // I: for<'all> InputSliceTy<'src, 'all, Sliced = &'all str>,
{
    type View = <I as InputSliceTy<'src, 'tmp>>::Sliced;

    fn fullmatch_v(&self, input: &'src mut I) -> Self::View {
        input.get_slice()
    }
}

pub trait Quattrn<'src, I: InputSlice<'src>>: for<'tmp> QuattrnTy<'src, 'tmp, I> {}

impl<'src, I, Q> Quattrn<'src, I> for Q
where
    I: InputSlice<'src>,
    Q: for<'tmp> QuattrnTy<'src, 'tmp, I>,
{
}

pub trait Pattern<'src, I: InputSlice<'src>> {
    type Captured;

    fn fullmatch(&self, input: &'src mut I) -> Self::Captured;
}

impl<'src, I, Q> Pattern<'src, I> for Q
where
    I: InputSlice<'src>,
    Q: Quattrn<'src, I>,
{
    type Captured = <Q as QuattrnTy<'src, 'src, I>>::View;

    fn fullmatch(&self, input: &'src mut I) -> Self::Captured {
        self.fullmatch_v(input)
    }
}

//------------------------------------------------------------------------------

pub fn q_string<'src, 'tmp>() -> impl for<'all> Quattrn<'src, String> {
    ()
}

pub fn p_string<'src>() -> impl Pattern<'src, String> {
    ()
}

pub fn q_str<'src>() -> impl Quattrn<'src, &'src str> {
    ()
}

// pub fn p_str<'src>() -> impl Pattern<'src, &'src str> {
//     ()
// }

#[test]
fn foo() {
    // let pat = p_string();
    // let mut s = String::new();
    // let a = pat.fullmatch(&mut s);

    let pat = q_str();
    let mut s = "";
    let a = pat.fullmatch(&mut s);
}

use super::*;

pub const fn behind<U>(optional: U::Item, required: U::Item)
where
    U: ?Sized + ThinSlice,
{
}

pub const fn behinds() {}

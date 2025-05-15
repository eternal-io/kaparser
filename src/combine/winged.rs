use super::*;

pub const fn winged<U>(primary: U::Item, secondary: U::Item)
where
    U: ?Sized + ThinSlice,
{
}

pub const fn winged_flipped<U>(outer: U::Item, inner: U::Item)
where
    U: ?Sized + ThinSlice,
{
}

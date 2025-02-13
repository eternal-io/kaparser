use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

#[doc(hidden)]
pub use paste::paste;

#[cold]
#[inline(always)]
pub(crate) const fn cold_path() {}

#[inline(always)]
pub(crate) const fn likely(cond: bool) -> bool {
    if !cond {
        cold_path();
    }
    cond
}
#[inline(always)]
pub(crate) const fn unlikely(cond: bool) -> bool {
    if cond {
        cold_path();
    }
    cond
}

//------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transfer {
    Accepted(usize),
    Rejected,
    Halt(usize),
}

impl Transfer {
    #[inline(always)]
    pub const fn perhaps(opt: Option<usize>) -> Self {
        if let Some(len) = opt {
            Self::Accepted(len)
        } else {
            Self::Rejected
        }
    }
}

//------------------------------------------------------------------------------

#[allow(clippy::len_without_is_empty)]
pub trait Slice {
    fn len(&self) -> usize;
    fn split_at(&self, mid: usize) -> (&Self, &Self);
}

impl Slice for str {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.split_at(mid)
    }
}

impl<T> Slice for [T] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.split_at(mid)
    }
}

//------------------------------------------------------------------------------

/// You can abbreviate `n..=n` to `n`.
pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn unsaturated(&self, times: usize) -> bool;
}

#[rustfmt::skip]
mod urange_bounds {
    use super::*;

    impl URangeBounds for usize {
        fn contains(&self, times: usize) -> bool { times == *self }
        fn unsaturated(&self, times: usize) -> bool { times <= *self }
    }
    impl URangeBounds for RangeFull {
        fn contains(&self, _t: usize) -> bool { true }
        fn unsaturated(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for RangeFrom<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unsaturated(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for Range<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unsaturated(&self, times: usize) -> bool { times < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unsaturated(&self, times: usize) -> bool { times < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unsaturated(&self, times: usize) -> bool { times <= *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unsaturated(&self, times: usize) -> bool { times <= self.end }
    }
}

//------------------------------------------------------------------------------

macro_rules! resume_proceed {
    (   $label:lifetime: $check:expr => {
        $( $LabelN:lifetime: $PointN:pat => $BlockN:block )+
    } ) => {
        resume_proceed!( @REARRANGE $label: $check => ; $($LabelN: $PointN => $BlockN)+ );
    };

    ( @REARRANGE $label:lifetime: $check:expr =>
        $( $LabelN:lifetime: $PointN:pat => $BlockN:block )* ;
           $LabelK:lifetime: $PointK:pat => $BlockK:block
        $( $LabelM:lifetime: $PointM:pat => $BlockM:block )*
    ) => {
        resume_proceed! {
            @REARRANGE $label: $check =>
              $LabelK: $PointK => $BlockK
            $($LabelN: $PointN => $BlockN)* ;
            $($LabelM: $PointM => $BlockM)*
        }
    };

    ( @REARRANGE $label:lifetime: $check:expr =>
        $( $LabelN:lifetime: $PointN:pat => $BlockN:block )+ ;
    ) => {
        resume_proceed!( @ENTER $label: $check => ; $($LabelN: $PointN => $BlockN)+ );
    };

    ( @ENTER $label:lifetime: $check:expr =>
        $( $LabelN:lifetime: $PointN:pat => $BlockN:block )* ;
           $LabelK:lifetime: $PointK:pat => $BlockK:block
        $( $LabelM:lifetime: $PointM:pat => $BlockM:block )+
    ) => {
        #[allow(unused_labels)]
        $LabelK: loop {
            resume_proceed! {
                @ENTER $label: $check =>
                $($LabelN: $PointN => $BlockN)*
                  $LabelK: $PointK => $BlockK ;
                $($LabelM: $PointM => $BlockM)+
            }
            $BlockK
            break;
        }
    };

    ( @ENTER $label:lifetime: $check:expr =>
        $( $LabelN:lifetime: $PointN:pat => $BlockN:block )* ;
           $LabelK:lifetime: $PointK:pat => $BlockK:block
    ) => {
        #[allow(unused_labels)]
        $LabelK: loop {
            resume_proceed!( @MATCH $label: $check => $($LabelN)* $LabelK $label ; $($PointN,)* $PointK );
            $BlockK
            break;
        }
    };

    ( @MATCH $label:lifetime: $check:expr =>
        $LabelA:lifetime $( $LabelN:lifetime )+ ;
                         $( $PointN:pat ),+
    ) => {
        resume_proceed!( @MATCH $label: $check => ; $($PointN => $LabelN)+ )
    };

    ( @MATCH $label:lifetime: $check:expr =>
        $( $PointN:pat => $LabelN:lifetime )* ;
           $PointK:pat => $LabelK:lifetime
        $( $PointM:pat => $LabelM:lifetime )*
    ) => {
        resume_proceed!( @MATCH $label: $check => $PointK => $LabelK $($PointN => $LabelN)* ; $($PointM => $LabelM)* )
    };

    ( @MATCH $label:lifetime: $check:expr =>
        $( $PointN:pat => $LabelN:lifetime )+ ;
    ) => {
        $label: loop {
            match $check {
                $( $PointN => break $LabelN, )+
            }
        }
    };
}

//------------------------------------------------------------------------------

/// `Lens1X` means `LenX - 1`. `Gen` means "Generic". Always `N < K < M`.
macro_rules! gen_product_types {
    (      $Lens1K:literal ~ $GenK:ident ~ $OrdK:tt
        $( $Lens1M:literal ~ $GenM:ident ~ $OrdM:tt )*
    ) => {
        gen_product_types! { @
              $Lens1K ~ $GenK ~ $OrdK ;
            $($Lens1M ~ $GenM ~ $OrdM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $GenN:ident ~ $OrdN:tt )+ ;
           $Lens1K:literal ~ $GenK:ident ~ $OrdK:tt
        $( $Lens1M:literal ~ $GenM:ident ~ $OrdM:tt )*
    ) => { $crate::common::paste! {
        #[derive(Debug, Default)]
        pub struct [<Product $Lens1K>]<$($GenN),*> { $(
           #[doc = "Value " $OrdN " of " $Lens1K "."]
            pub [<val $OrdN>]: $GenN,
        )+ }

        gen_product_types! { @
            $($Lens1N ~ $GenN ~ $OrdN)+
              $Lens1K ~ $GenK ~ $OrdK ;
            $($Lens1M ~ $GenM ~ $OrdM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $GenN:ident ~ $OrdN:tt )+ ; ) => {};
}

// gen_product_types! {
//     0  ~ A ~ 1
//     1  ~ B ~ 2
//     2  ~ C ~ 3
//     3  ~ D ~ 4
//     4  ~ E ~ 5
//     5  ~ F ~ 6
//     6  ~ G ~ 7
//     7  ~ H ~ 8
//     8  ~ I ~ 9
//     9  ~ J ~ 10
//     10 ~ K ~ 11
//     11 ~ L ~ 12
//     12 ~ M ~ 13
//     13 ~ N ~ 14
//     14 ~ O ~ 15
//     15 ~ P ~ 16
//     16 ~ Q ~ 17
// }

//------------------------------------------------------------------------------

/// `Lens1X` means `LenX - 1`. `Gen` means "Generic". Always `N < K < M`.
macro_rules! gen_alternates {
    (      $Lens1K:literal ~ $GenK:ident ~ $OrdK:tt
        $( $Lens1M:literal ~ $GenM:ident ~ $OrdM:tt )*
    ) => {
        gen_alternates! { @
              $Lens1K ~ $GenK ~ $OrdK ;
            $($Lens1M ~ $GenM ~ $OrdM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $GenN:ident ~ $OrdN:tt )+ ;
           $Lens1K:literal ~ $GenK:ident ~ $OrdK:tt
        $( $Lens1M:literal ~ $GenM:ident ~ $OrdM:tt )*
    ) => { $crate::common::paste! {
        #[derive(Debug, Clone)]
        pub enum [<Alt $Lens1K>]<$($GenN),*> { $(
           #[doc = "Variant " $OrdN " of " $Lens1K "."]
            [<Var $OrdN>]($GenN),
        )+ }

        gen_alternates! { @
            $($Lens1N ~ $GenN ~ $OrdN)+
              $Lens1K ~ $GenK ~ $OrdK ;
            $($Lens1M ~ $GenM ~ $OrdM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $GenN:ident ~ $OrdN:tt )+ ; ) => {};
}

gen_alternates! {
    0  ~ A ~ 1
    1  ~ B ~ 2
    2  ~ C ~ 3
    3  ~ D ~ 4
    4  ~ E ~ 5
    5  ~ F ~ 6
    6  ~ G ~ 7
    7  ~ H ~ 8
    8  ~ I ~ 9
    9  ~ J ~ 10
    10 ~ K ~ 11
    11 ~ L ~ 12
    12 ~ M ~ 13
    13 ~ N ~ 14
    14 ~ O ~ 15
    15 ~ P ~ 16
    16 ~ Q ~ 17
}

//------------------------------------------------------------------------------

macro_rules! gen_checkpoints {
    (      $Lens1K:literal ~ $OrdK:tt
        $( $Lens1M:literal ~ $OrdM:tt )*
    ) => {
        gen_checkpoints! { @
              $Lens1K ~ $OrdK ;
            $($Lens1M ~ $OrdM)*
        }
    };

    ( @ $( $Lens1N:literal ~ $OrdN:tt )+ ;
           $Lens1K:literal ~ $OrdK:tt
        $( $Lens1M:literal ~ $OrdM:tt )*
    ) => { $crate::common::paste! {
        #[doc(hidden)]
        #[derive(Clone)]
        pub enum [<Check $Lens1K>] { $(
            [<Point $OrdN>],
        )+ }

        gen_checkpoints! { @
            $($Lens1N ~ $OrdN)+
              $Lens1K ~ $OrdK ;
            $($Lens1M ~ $OrdM)*
        }
    } };

    ( @ $( $Lens1N:literal ~ $OrdN:tt )+ ; ) => {};
}

gen_checkpoints! {
    0  ~ 1
    1  ~ 2
    2  ~ 3
    3  ~ 4
    4  ~ 5
    5  ~ 6
    6  ~ 7
    7  ~ 8
    8  ~ 9
    9  ~ 10
    10 ~ 11
    11 ~ 12
    12 ~ 13
    13 ~ 14
    14 ~ 15
    15 ~ 16
    16 ~ 17
}

use core::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

#[doc(hidden)]
pub use paste::paste;

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
#[cold]
#[inline(always)]
pub(crate) const fn cold_path() {}

//------------------------------------------------------------------------------

pub trait Slice {
    type Item: Copy + PartialEq;

    fn len(&self) -> usize;
    fn len_of(&self, item: Self::Item) -> usize;
    fn starts_with(&self, prefix: &Self) -> bool;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn split_at(&self, mid: usize) -> (&Self, &Self);
    fn iter(&self) -> impl Iterator<Item = Self::Item>;
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)>;
    fn first(&self) -> Option<Self::Item> {
        self.iter().next()
    }
}

impl Slice for str {
    type Item = char;

    #[inline(always)]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline(always)]
    fn len_of(&self, item: Self::Item) -> usize {
        item.len_utf8()
    }
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        (*self).starts_with(prefix)
    }

    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        self.chars()
    }
    #[inline(always)]
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)> {
        self.char_indices()
    }
}

impl<T> Slice for [T]
where
    T: Copy + PartialEq,
{
    type Item = T;

    #[inline(always)]
    fn len(&self) -> usize {
        (*self).len()
    }
    #[inline(always)]
    fn len_of(&self, _tem: Self::Item) -> usize {
        1
    }
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        (*self).starts_with(prefix)
    }

    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        (*self).split_at(mid)
    }
    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = Self::Item> {
        (*self).iter().copied()
    }
    #[inline(always)]
    fn iter_indices(&self) -> impl Iterator<Item = (usize, Self::Item)> {
        (*self).iter().copied().enumerate()
    }
}

//------------------------------------------------------------------------------

/// You can abbreviate `n..=n` to `n`.
pub trait URangeBounds {
    fn contains(&self, times: usize) -> bool;
    fn unfulfilled(&self, times: usize) -> bool;
}

#[rustfmt::skip]
mod urange_bounds {
    use super::*;

    impl URangeBounds for usize {
        fn contains(&self, times: usize) -> bool { times == *self }
        fn unfulfilled(&self, times: usize) -> bool { times < *self }
    }
    impl URangeBounds for RangeFull {
        fn contains(&self, _t: usize) -> bool { true }
        fn unfulfilled(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for RangeFrom<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, _t: usize) -> bool { true }
    }
    impl URangeBounds for Range<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times + 1 < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times < *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times < self.end }
    }
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! __resume_advance {
    (      $Ent:expr ;
        $( $CaseN:pat => $TurnN:block $ProcN:block )+
    ) => {
        __resume_advance! {
            @LABELING $Ent ;
            'p1  'p2  'p3  'p4  'p5  'p6  'p7  'p8
            'p9  'p10 'p11 'p12 'p13 'p14 'p15 'p16
            'p17 'p18 'p19 'p20 'p21 'p22 'p23 'p24
            'p25 'p26 'p27 'p28 'p29 'p30 'p31 'p32 ;
            $( $CaseN => $TurnN $ProcN )+ ;
        }
    };

    ( @LABELING $Ent:expr ;
           $LabK:lifetime
        $( $LabM:lifetime )* ;
           $CaseK:pat => $TurnK:block $ProcK:block
        $( $CaseM:pat => $TurnM:block $ProcM:block )* ;
        $( $LabN:lifetime:
           $CaseN:pat => $TurnN:block $ProcN:block )*
    ) => {
        __resume_advance! {
            @LABELING $Ent ;
            $( $LabM )* ;
            $( $CaseM => $TurnM $ProcM )* ;
               $LabK:
               $CaseK => $TurnK $ProcK // cases then appear in reverse order.
            $( $LabN:
               $CaseN => $TurnN $ProcN )*
        }
    };

    ( @LABELING $Ent:expr ;
        /* not enough labels */ ;
        $CaseX:pat => $( $tt:tt )*
    ) => {
        ::core::compile_error!("too many cases, only 32 at most")
    };

    ( @LABELING $Ent:expr ;
        $( $LabN:lifetime )* ;
        /* no more unlabeled cases */ ;
        $( $tt:tt )*
    ) => {
        __resume_advance! { @ENTERING $Ent ; ; $( $tt )* }
    };

    ( @ENTERING $Ent:expr ;
        $( $LabN:lifetime: $CaseN:pat => $TurnN:block $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $TurnK:block $ProcK:block
        $( $LabM:lifetime: $CaseM:pat => $TurnM:block $ProcM:block )+
    ) => {
        $LabK: {
            __resume_advance! {
                @ENTERING $Ent ;
                   $LabK: $CaseK => $TurnK $ProcK // reverse again, but not so important.
                $( $LabN: $CaseN => $TurnN $ProcN )* ;
                $( $LabM: $CaseM => $TurnM $ProcM )+
            }
            $TurnK
        }
        $ProcK
    };

    ( @ENTERING $Ent:expr ;
        $( $LabN:lifetime: $CaseN:pat => $TurnN:block $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $TurnK:block $ProcK:block
    ) => {
        $LabK: {
            match $Ent {
                $CaseK => break $LabK,
              $($CaseN => break $LabN,)*
            }
        }
        $ProcK
    }
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
    ) => { paste::paste! {
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

//------------------------------------------------------------------------------

pub(crate) use alts::*;

pub mod alts {
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
        ) => { paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
            pub enum [<Alt $Lens1K>]<$($GenN),+> { $(
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
}

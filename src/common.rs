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

/// `Lens1X` means `LenX - 1`. Always `N < K < M`.
/// `Gen` means "Generic". "Con" means "Converge".
macro_rules! __generate_codes {
    ( $callback:ident $(($($custom:ident) ~ +))? ) => { paste::paste! {
        __generate_codes! {
          @ $callback ;
            0  ~ 1  $(~ ($([< $custom 1  >]) ~ +))? ~ A ~ X ~ 0
            1  ~ 2  $(~ ($([< $custom 2  >]) ~ +))? ~ B ~ X ~ 1
            2  ~ 3  $(~ ($([< $custom 3  >]) ~ +))? ~ C ~ X ~ 2
            3  ~ 4  $(~ ($([< $custom 4  >]) ~ +))? ~ D ~ X ~ 3
            4  ~ 5  $(~ ($([< $custom 5  >]) ~ +))? ~ E ~ X ~ 4
            5  ~ 6  $(~ ($([< $custom 6  >]) ~ +))? ~ F ~ X ~ 5
            6  ~ 7  $(~ ($([< $custom 7  >]) ~ +))? ~ G ~ X ~ 6
            7  ~ 8  $(~ ($([< $custom 8  >]) ~ +))? ~ H ~ X ~ 7
            8  ~ 9  $(~ ($([< $custom 9  >]) ~ +))? ~ I ~ X ~ 8
            9  ~ 10 $(~ ($([< $custom 10 >]) ~ +))? ~ J ~ X ~ 9
            10 ~ 11 $(~ ($([< $custom 11 >]) ~ +))? ~ K ~ X ~ 10
            11 ~ 12 $(~ ($([< $custom 12 >]) ~ +))? ~ L ~ X ~ 11
            12 ~ 13 $(~ ($([< $custom 13 >]) ~ +))? ~ M ~ X ~ 12
            13 ~ 14 $(~ ($([< $custom 14 >]) ~ +))? ~ N ~ X ~ 13
            14 ~ 15 $(~ ($([< $custom 15 >]) ~ +))? ~ O ~ X ~ 14
            15 ~ 16 $(~ ($([< $custom 16 >]) ~ +))? ~ P ~ X ~ 15
            16 ~ 17 $(~ ($([< $custom 17 >]) ~ +))? ~ Q ~ X ~ 16
            17 ~ 18 $(~ ($([< $custom 18 >]) ~ +))? ~ R ~ X ~ 17
            18 ~ 19 $(~ ($([< $custom 19 >]) ~ +))? ~ S ~ X ~ 18
            19 ~ 20 $(~ ($([< $custom 20 >]) ~ +))? ~ T ~ X ~ 19
            20 ~ 21 $(~ ($([< $custom 21 >]) ~ +))? ~ U ~ X ~ 20
            21 ~ 22 $(~ ($([< $custom 22 >]) ~ +))? ~ V ~ X ~ 21
            22 ~ 23 $(~ ($([< $custom 23 >]) ~ +))? ~ W ~ X ~ 22
        }
    } };

    ( @ $callback:ident ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        __generate_codes! {
          @ $callback ;
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
        $Lens1K:literal ~ $OrdK:literal $(~ ($($CusK:ident) ~ +))? ~ $GenK:ident ~ $ConK:ident ~ $IdxK:tt
      $($Lens1M:literal ~ $OrdM:literal $(~ ($($CusM:ident) ~ +))? ~ $GenM:ident ~ $ConM:ident ~ $IdxM:tt)*
    ) => {
        $callback!( $Lens1K, $($OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+ );
        __generate_codes! {
          @ $callback ;
          $($Lens1N ~ $OrdN $(~ ($($CusN) ~ +))? ~ $GenN ~ $ConN ~ $IdxN)+
            $Lens1K ~ $OrdK $(~ ($($CusK) ~ +))? ~ $GenK ~ $ConK ~ $IdxK ;
          $($Lens1M ~ $OrdM $(~ ($($CusM) ~ +))? ~ $GenM ~ $ConM ~ $IdxM)*
        }
    };

    ( @ $callback:ident ;
      $($Lens1N:literal ~ $OrdN:literal $(~ ($($CusN:ident) ~ +))? ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ;
    ) => {};
}

//------------------------------------------------------------------------------

pub(crate) use alts::*;
pub(crate) use checkpoints::*;

pub mod alts {
    macro_rules! gen_alternative {
        ( $Len:literal, $($OrdN:literal ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ) => { paste::paste! {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
            pub enum [<Alt $Len>]<$($GenN),+> { $(
            #[doc = "Variant " $OrdN " of " $Len "."]
                [<Var $OrdN>]($GenN),
            )+ }
        } }
    }

    __generate_codes! { gen_alternative }
}

#[doc(hidden)]
pub mod checkpoints {
    macro_rules! gen_checkpoint {
        ( $Len:literal, $($OrdN:literal ~ $GenN:ident ~ $ConN:ident ~ $IdxN:tt)+ ) => { paste::paste! {
            #[doc(hidden)]
            #[derive(Clone)]
            pub enum [<Check $Len>] { $(
                [<Point $OrdN>],
            )+ }
        } }
    }

    __generate_codes! { gen_checkpoint }
}

//------------------------------------------------------------------------------

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

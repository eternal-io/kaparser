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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transfer {
    Accepted,
    Rejected,
    Halt,
}

impl Transfer {
    #[inline(always)]
    pub const fn is_accepted(self) -> bool {
        match self {
            Transfer::Accepted => true,
            Transfer::Rejected | Transfer::Halt => false,
        }
    }

    #[inline(always)]
    pub const fn perhaps(res: Result<usize, usize>) -> (Self, usize) {
        match res {
            Ok(len) => (Self::Accepted, len),
            Err(len) => (Self::Rejected, len),
        }
    }

    #[inline(always)]
    pub const fn cut(self) -> Self {
        match self {
            Self::Accepted => Self::Accepted,
            Self::Rejected | Self::Halt => Self::Halt,
        }
    }
}

//------------------------------------------------------------------------------

pub trait Slice {
    fn len(&self) -> usize;
    fn split_at(&self, mid: usize) -> (&Self, &Self);
    fn starts_with(&self, prefix: &Self) -> bool;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
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
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        self.starts_with(prefix)
    }
}

impl<T: PartialEq> Slice for [T] {
    #[inline(always)]
    fn len(&self) -> usize {
        self.len()
    }
    #[inline(always)]
    fn split_at(&self, mid: usize) -> (&Self, &Self) {
        self.split_at(mid)
    }
    #[inline(always)]
    fn starts_with(&self, prefix: &Self) -> bool {
        self.starts_with(prefix)
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
        fn unfulfilled(&self, times: usize) -> bool { times <= *self }
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
        fn unfulfilled(&self, times: usize) -> bool { times < self.end }
    }
    impl URangeBounds for RangeTo<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times < self.end }
    }
    impl URangeBounds for RangeInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times <= *self.end() }
    }
    impl URangeBounds for RangeToInclusive<usize> {
        fn contains(&self, times: usize) -> bool { self.contains(&times) }
        fn unfulfilled(&self, times: usize) -> bool { times <= self.end }
    }
}

//------------------------------------------------------------------------------

macro_rules! resume_precede {
    (
        $switch:expr => {
            $( $LabN:lifetime: $CaseN:pat => $([$InitN:block])? $ProcN:block )+
        }
    ) => {
        resume_precede!( @REARRANGE $switch => ; $($LabN: $CaseN => $([$InitN])? $ProcN)+ );
    };

    ( @REARRANGE $switch:expr =>
        $( $LabN:lifetime: $CaseN:pat => $([$InitN:block])? $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $([$InitK:block])? $ProcK:block
        $( $LabM:lifetime: $CaseM:pat => $([$InitM:block])? $ProcM:block )*
    ) => {
        resume_precede! {
            @REARRANGE $switch =>
              $LabK: $CaseK => $([$InitK])? $ProcK
            $($LabN: $CaseN => $([$InitN])? $ProcN)* ;
            $($LabM: $CaseM => $([$InitM])? $ProcM)*
        }
    };

    ( @REARRANGE $switch:expr =>
        $( $LabN:lifetime: $CaseN:pat => $([$InitN:block])? $ProcN:block )+ ;
    ) => {
        resume_precede!( @ENTER $switch => ; $($LabN: $CaseN => $([$InitN])? $ProcN)+ );
    };

    ( @ENTER $switch:expr =>
        $( $LabN:lifetime: $CaseN:pat => $([$InitN:block])? $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $([$InitK:block])? $ProcK:block
        $( $LabM:lifetime: $CaseM:pat => $([$InitM:block])? $ProcM:block )+
    ) => {
        $LabK: loop {
            resume_precede! {
                @ENTER $switch =>
                  $LabK: $CaseK => $([$InitK])? $ProcK
                $($LabN: $CaseN => $([$InitN])? $ProcN)* ;
                $($LabM: $CaseM => $([$InitM])? $ProcM)+
            }
            $($InitK)?
            break;
        }
        $ProcK
    };

    ( @ENTER $switch:expr =>
        $( $LabN:lifetime: $CaseN:pat => $([$InitN:block])? $ProcN:block )* ;
           $LabK:lifetime: $CaseK:pat => $([$InitK:block])? $ProcK:block
    ) => {
        $LabK: loop {
            match $switch {
                $CaseK => break $LabK,
              $($CaseN => break $LabN,)*
            }
        }
        $ProcK
    };
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
        #[doc(hidden)]
        #[derive(Debug, Clone)]
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

macro_rules! gen_checkpoint_enumerations {
    (      $Kn:literal ~ $kth:literal
        $( $Ln:literal ~ $nth:literal )*
    ) => {
        gen_checkpoint_enumerations!( @ $Kn ~ $kth ; $($Ln ~ $nth)* );
    };

    ( @ $( $Ln:literal ~ $nth:literal )+ ;
           $Lk:literal ~ $kth:literal
        $( $Lm:literal ~ $mth:literal )* ) => { $crate::paste! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum [<Checkpoint $Lk s>] { $(
            #[doc = "Checkpoint " $nth " of " $Lk "."]
            [<_ $nth>],
        )+ }

        gen_checkpoint_enumerations!( @ $($Ln ~ $nth)+ $Lk ~ $kth ; $($Lm ~ $mth)* );
    } };

    ( @ $( $Ln:literal ~ $nth:literal )+ ; ) => {};
}

gen_checkpoint_enumerations! {
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

macro_rules! gen_alternate_discriminants {
    (      $Lk:literal ~ $Kth:ident ~ $kth:literal
        $( $Ln:literal ~ $Nth:ident ~ $nth:literal )*
    ) => {
        gen_alternate_discriminants!( @ $Lk ~ $Kth ~ $kth ; $($Ln ~ $Nth ~ $nth)* );
    };

    ( @ $( $Ln:literal ~ $Nth:ident ~ $nth:literal )+ ;
           $Lk:literal ~ $Kth:ident ~ $kth:literal
        $( $Lm:literal ~ $Mth:ident ~ $mth:literal )*
    ) => { $crate::paste! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum [<Alternate $Lk s>]<$([<A $nth>]),+> { $(
            #[doc = "Alternate " $nth " of " $Lk "."]
            $Nth([<A $nth>]),
        )+ }

        gen_alternate_discriminants! { @
            $($Ln ~ $Nth ~ $nth)+
              $Lk ~ $Kth ~ $kth ;
            $($Lm ~ $Mth ~ $mth)*
        }
    } };

    ( @ $( $Ln:literal ~ $Nth:ident ~ $nth:literal )+ ; ) => {};
}

gen_alternate_discriminants! {
    0  ~ First      ~ 1
    1  ~ Second     ~ 2
    2  ~ Third      ~ 3
    3  ~ Fourth     ~ 4
    4  ~ Fifth      ~ 5
    5  ~ Sixth      ~ 6
    6  ~ Seventh    ~ 7
    7  ~ Eighth     ~ 8
    8  ~ Ninth      ~ 9
    9  ~ Tenth      ~ 10
    10 ~ Eleventh   ~ 11
    11 ~ Twelth     ~ 12
    12 ~ Thirteenth ~ 13
    13 ~ Fourteenth ~ 14
    14 ~ Fifteenth  ~ 15
    15 ~ Sixteenth  ~ 16
    16 ~ DONT_CARE  ~ 17
}

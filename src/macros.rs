/// Combine predicates, produce a new predicate that accepts only these specified characters.
#[doc(hidden)]
#[macro_export]
macro_rules! all {
    ( $($preds:expr),+ $(,)? ) => {
        move |ch: &char| all!( @ ch $($preds),+ )
    };

    ( @ $ch:ident $pred:expr, $($preds:expr),* ) => {
        $pred.predicate($ch) || all!( @ $ch $($preds),* )
    };

    ( @ $ch:ident $pred:expr ) => {
        $pred.predicate($ch)
    };
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! alt {
    ( $($p:expr),* $(,)? ) => {
        $crate::combine::alt::alternative::<_, _, _>(($($p,)*))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! com {
    ( $($p:expr),* $(,)? ) => {
        $crate::combine::com::compound::<_, _, _>(($($p,)*))
    };
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! len {
    ($n:expr, $p:expr) => {
        $crate::combine::lens::lens::<_, _, _, { $n }>($p)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! rep {
    ($n:tt, $p:expr) => {
        $crate::combine::repeat::repeat_exact::<_, _, _, { $n }>($p)
    };
    ($n:tt..=$m:tt, $p:expr) => {
        $crate::combine::repeat::repeat::<_, _, _, { $n }, { $m - $n }>($p)
    };
    (..=$m:tt, $p:expr) => {
        $crate::combine::repeat::repeat_at_most::<_, _, _, { $m }>($p)
    };

    ($n:tt..$m:tt, $p:expr) => {
        ::core::compile_error!("use `n..=m` instead")
    };
    (..$m:tt, $p:expr) => {
        ::core::compile_error!("use `..=m` instead")
    };

    ($n:tt.., $p:expr) => {
        ::core::compile_error!("consider use `reiter` instead")
    };
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! rules {
    ( $(#[$attr:meta])*
        struct $name:ident<$sli:ty> {
            $($(#[$bttr:meta])*
                $field:ident: $pat:expr
            ),* $(,)?
        }
      $($rest:tt)*
    ) => {};

    ( $(#[$attr:meta])*
        enum $name:ident<$sli:ty> {
            $($(#[$bttr:meta])*
                $discr:ident = $pat:expr
            ),+ $(,)?
        }
      $($rest:tt)*
    ) => { $crate::common::paste! {
      $(#[$attr])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) enum $name { $(
          $(#[$bttr])*
            $discr($pat::Captured),
        )+ }

        pub(crate) struct [<$name T>]; // TODO: doc

        #[doc(hidden)]
        enum [<$name __>] { $(
            $discr($pat::Internal),
        )+ }

        impl<'i, E> Pattern<'i, $sli, E> for [<$name T>]
        where
            E: $crate::error::Situation,
        {
            type Captured = $name;
            type Internal = [<$name __>];

            #[inline(always)]
            fn init(&self) -> Self::Internal {
                rules! { @init_alt [<$name __>] $($discr = $pat,)+ }
            }

            #[inline(always)]
            fn advance(&self, slice: &U, entry: &mut Self::Internal, eof: bool) -> ::core::result::Result<usize, E> {
                use [<$name __>]::*;

                resume_advance! {
                    entry => { $(

                    )+ }
                }
            }

            #[inline(always)]
            fn extract(&self, slice: &'i U, entry: Self::Internal) -> Self::Captured {
                use [<$name __>]::*;
                match entry { $(
                    $discr(state) => $name::$discr($pat.extract(slice, state)),
                )+ }
            }
        }
    } };

    () => {};

    (   @init_alt
        $name__:ident
        $discr0:ident = $pat0:expr,
     $( $discrs:ident = $pats:expr, )*
    ) => {
        $name__::$discr0($pat0::init())
    };
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! define_alternative {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> --
        $($(#[$bttr:meta])*
            $discr:ident = $patt:expr
        ),* $(,)?
    ) => { $crate::common::paste! {
        // TODO!
    } }
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! define_dispatch {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> --
        $($(#[$bttr:meta])*
            $discr:ident = $head:expr => $body:expr
        ),* $(,)?
    ) => { $crate::common::paste! {
        // TODO!
    } }
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! define_sequence {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> --
        $($(#[$bttr:meta])*
            $field:ident: $patt:expr
        ),* $(,)?
    ) => { $crate::common::paste! {
        // TODO!
    } }
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! define_compound {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> --
      $($patt:expr),* $(,)?
    ) => { $crate::common::paste! {
        // TODO!
    } }
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! define_token_set {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> --
        $($(#[$bttr:meta])*
            $discr:ident = $token:expr
        ),* $(,)?
    ) => { $crate::common::paste! {
      $(#[$attr])*
        #[doc = "\n\n*(generated token set discriminant)*"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) enum $name { $(
          $(#[$bttr])*
            #[doc = "\n\nAssociates `` " $token " ``"]
            $discr,
        )* }

        #[doc = "\n\nGenerated token set pattern with [`" $name "`] discriminant."]
        pub(crate) struct [<$name T>];

        impl<'i, E> $crate::pattern::Pattern<'i, $sli, E> for [<$name T>]
        where
            E: $crate::error::Situation,
        {
            type Captured = $name;
            type Internal = ::core::option::Option<$name>;

            #[inline(always)]
            fn init(&self) -> Self::Internal {
                ::core::option::Option::None
            }

            #[inline(always)]
            fn advance(&self, slice: &$sli, entry: &mut Self::Internal, eof: bool) -> ::core::result::Result<usize, E> {
                use ::core::prelude::v1::*;

                let max_len = const { define_token_set!( @MAX $($token.len(),)* 0 ) };

                if !eof && slice.len() < max_len {
                    return E::raise_unfulfilled(Some((max_len - slice.len()).try_into().unwrap()));
                }
            $(
                if slice.starts_with($token) {
                    *entry = Some($name::$discr);
                    return const { Ok($token.len()) };
                }
            )*
                E::raise_reject_at(0)
            }

            #[inline(always)]
            fn extract(&self, _lice: &'i $sli, entry: Self::Internal) -> Self::Captured {
                entry.unwrap()
            }
        }
    } };

    ( @MAX $expr:expr ) => { $expr };

    ( @MAX $expr:expr, $($exprs:expr),+ ) => { {
        let a = $expr;
        let b = define_token_set!( @MAX $($exprs),+ );

        if a > b { a } else { b }
    } };
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    define_token_set! {
        Boolean<str> --
        False = "false",
        True = "true",
    }

    #[test]
    fn tkst() {
        let pat = simple_opaque(BooleanT);
        assert_eq!(pat.full_match("true").unwrap(), Boolean::True);
        assert_eq!(pat.full_match("false").unwrap(), Boolean::False);
        assert!(pat.full_match("False").is_err());
    }
}

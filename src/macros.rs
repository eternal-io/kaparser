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
macro_rules! token_set {
    ( $(#[$attr:meta])*
        $name:ident<$sli:ty>;
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

        #[allow(non_upper_case_globals)]
        impl [<$name T>] {
        $(
            const $discr: &$sli = $token;
        )*
            const fn max_len() -> usize {
                token_set!( @MAX $($token.len(),)* 0 )
            }
        }

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
                if !eof && slice.len() < Self::max_len() {
                    return E::raise_unfulfilled((Self::max_len() - slice.len()).try_into().ok());
                }
            $(
                if slice.starts_with(Self::$discr) {
                    *entry = ::core::option::Option::Some($name::$discr);
                    return ::core::result::Result::Ok(Self::$discr.len());
                }
            )*
                E::raise_reject_at(0)
            }

            #[inline(always)]
            fn extract(&self, _lice: &'i $sli, entry: Self::Internal) -> Self::Captured {
                entry.expect("contract violation")
            }
        }
    } };

    ( @MAX $expr:expr ) => { $expr };

    ( @MAX $expr:expr, $($exprs:expr),+ ) => { {
        let a = $expr;
        let b = token_set!( @MAX $($exprs),+ );

        if a > b { a } else { b }
    } };
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    const TK_TRUE: &str = "true";
    const TK_FALSE: &str = "false";

    token_set! {
        Boolean<str>;
        False = TK_FALSE,
        True = TK_TRUE,
    }

    #[test]
    fn tkst() {
        let pat = simple_opaque(BooleanT);
        assert_eq!(pat.full_match("true").unwrap(), Boolean::True);
        assert_eq!(pat.full_match("false").unwrap(), Boolean::False);
        assert!(pat.full_match("False").is_err());
    }
}

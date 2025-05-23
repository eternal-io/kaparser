#[doc(hidden)]
#[macro_export]
macro_rules! len {
    ($n:expr, $patt:expr) => {
        $crate::combine::lens::lens::<_, { $n }>($patt)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! rep {
    ($n:tt, $patt:expr) => {
        $crate::combine::repeat::repeat_exact::<_, _, _, { $n }>($patt)
    };
    ($n:tt..=$m:tt, $patt:expr) => {
        $crate::combine::repeat::repeat::<_, _, _, { $n }, { $m - $n }>($patt)
    };
    (..=$m:tt, $patt:expr) => {
        $crate::combine::repeat::repeat_at_most::<_, _, _, { $m }>($patt)
    };

    ($n:tt..$m:tt, $patt:expr) => {
        ::core::compile_error!("use `n..=m` instead")
    };
    (..$m:tt, $patt:expr) => {
        ::core::compile_error!("use `..=m` instead")
    };

    ($n:tt.., $patt:expr) => {
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

            #[inline]
            fn init(&self) -> Self::Internal {
                ::core::option::Option::None
            }

            #[inline]
            #[allow(unused_variables)]
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

            #[inline]
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
        Unused<str>;
    }
    token_set! {
        Boolean<str>;
        False = TK_FALSE,
        True = TK_TRUE,
    }

    #[test]
    fn tkst() {
        let pat = impls::opaque_simple(BooleanT);
        assert_eq!(pat.fullmatch("true").unwrap(), Boolean::True);
        assert_eq!(pat.fullmatch("false").unwrap(), Boolean::False);
        assert!(pat.fullmatch("False").is_err());
    }
}

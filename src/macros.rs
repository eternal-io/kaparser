#[doc(hidden)]
#[macro_export]
macro_rules! len {
    ($n:expr, $p:expr) => {
        $crate::combine::lens::lens::<_, _, _, { $n }>($p)
    };
}

//------------------------------------------------------------------------------

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
        compile_error!("use `n..=m` instead")
    };
    (..$m:tt, $p:expr) => {
        compile_error!("use `..=m` instead")
    };

    ($n:tt.., $p:expr) => {
        compile_error!("not supported yet")
    };
}

//------------------------------------------------------------------------------

/// Generate structures, implement [`Pattern`](crate::pattern::Pattern) for a single token conveniently.
#[doc(hidden)]
#[macro_export]
macro_rules! token {
    () => {};
}

//------------------------------------------------------------------------------

/// Generate structures, implement [`Pattern`](crate::pattern::Pattern) for a set of tokens conveniently.
#[doc(hidden)]
#[macro_export]
macro_rules! token_set {
    ( $(
        $(#[$attr:meta])*
        $name:ident<$sli:ty> { $(
            $(#[$bttr:meta])*
            $key:ident = $word:literal
        ),* $(,)? }
    )* ) => { $( $crate::common::paste! {
      $(#[$attr])*
        #[doc = "\n\n*(generated token discriminant)*"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) enum $name { $(
          $(#[$bttr])*
            #[doc = "\n\nAssociates `` " $word " ``"]
            $key,
        )* }

        #[doc = "Generated tokens pattern with [`" $name "`] discriminant."
                "\n\nZST type by [`token_set!`] macro."]
        pub(crate) struct [<$name T>];

        impl<'i, E> Pattern<'i, $sli, E> for [<$name T>]
        where
            E: $crate::error::Situation,
        {
            type Captured = $name;
            type Internal = Option<$name>;

            #[inline(always)]
            fn init(&self) -> Self::Internal {
                None
            }

            #[inline(always)]
            fn precede(&self, slice: &$sli, entry: &mut Self::Internal, eof: bool) -> Result<usize, E> {
                let max_len = const { token_set!( @max $($word.len(),)* 0 ) };
                if !eof && slice.len() < max_len {
                    return E::raise_unfulfilled(Some((max_len - slice.len()).try_into().unwrap()));
                }
            $(
                if slice.starts_with($word) {
                    *entry = Some($name::$key);
                    return const { Ok($word.len()) };
                }
            )*
                E::raise_reject_at(0)
            }

            #[inline(always)]
            fn extract(&self, _lice: &'i $sli, entry: Self::Internal) -> Self::Captured {
                entry.unwrap()
            }
        }
    } )* };

    ( @max $expr:expr ) => { $expr };

    ( @max $expr:expr, $( $exprs:expr ),+ ) => {{
        let a = $expr;
        let b = token_set!( @max $($exprs),+ );

        if a > b { a } else { b }
    }};
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    token_set! {
        Boolean<str> {
            False = "false",
            True = "true",
        }
    }

    #[test]
    fn tk() {}

    #[test]
    fn tkst() {
        let pat = __pat::<_, _, SimpleError>(BooleanT);
        assert_eq!(pat.full_match("true").unwrap(), Boolean::True);
        assert_eq!(pat.full_match("false").unwrap(), Boolean::False);
        assert!(pat.full_match("False").is_err());
    }
}

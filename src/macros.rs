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
        $crate::combine::alt::alternate::<_, _, _>(($($p,)*))
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
macro_rules! tokens {
    ( $(#[$attr:meta])*
        $name:ident: $sli:ty = $lit:literal;
      $($rest:tt)*
    ) => { $crate::common::paste! {
      $(#[$attr:meta])*
        #[doc = "\n\nGenerated token pattern, associates `` " $lit " ``"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) struct $name;

        impl<'i, E> Pattern<'i, $sli, E> for $name
        where
            E: $crate::error::Situation,
        {
            type Captured = Self;
            type Internal = ();

            #[inline(always)]
            fn init(&self) -> Self::Internal {
                ()
            }

            #[inline(always)]
            fn advance(&self, slice: &$sli, _ntry: &mut Self::Internal, eof: bool) -> ::core::result::Result<usize, E> {
                use $crate::common::Slice;

                let the_len = const { $lit.len() };
                if slice.len() < the_len {
                    match eof {
                        true => E::raise_reject_at(slice.len()),
                        false => E::raise_unfulfilled(Some((the_len - slice.len()).try_into().unwrap())),
                    }
                } else {
                    for ((off, expected), item) in $lit.iter_indices().zip(slice.iter()) {
                        if item != expected {
                            return E::raise_reject_at(off);
                        }
                    }
                    Ok(the_len)
                }
            }

            #[inline(always)]
            fn extract(&self, _lice: &'i $sli, _ntry: Self::Internal) -> Self::Captured {
                Self
            }
        }

        tokens! { $($rest)* }
    } };

    ( $(#[$attr:meta])*
        $name:ident<$sli:ty> {
            $($(#[$bttr:meta])*
                $discr:ident = $lit:literal
            ),* $(,)?
        }
      $($rest:tt)*
    ) => { $crate::common::paste! {
      $(#[$attr])*
        #[doc = "\n\n*(generated token discriminant)*"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub(crate) enum $name { $(
          $(#[$bttr])*
            #[doc = "\n\nAssociates `` " $lit " ``"]
            $discr,
        )* }

        #[doc = "\n\nGenerated tokens pattern with [`" $name "`] discriminant."]
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
            fn advance(&self, slice: &$sli, entry: &mut Self::Internal, eof: bool) -> ::core::result::Result<usize, E> {
                let max_len = const { tokens!( @max $($lit.len(),)* 0 ) };
                if !eof && slice.len() < max_len {
                    return E::raise_unfulfilled(Some((max_len - slice.len()).try_into().unwrap()));
                }
            $(
                if slice.starts_with($lit) {
                    *entry = Some($name::$discr);
                    return const { Ok($lit.len()) };
                }
            )*
                E::raise_reject_at(0)
            }

            #[inline(always)]
            fn extract(&self, _lice: &'i $sli, entry: Self::Internal) -> Self::Captured {
                entry.unwrap()
            }
        }

        tokens! { $($rest)* }
    } };

    () => {};

    ( @max $expr:expr ) => { $expr };

    ( @max $expr:expr, $( $exprs:expr ),+ ) => {{
        let a = $expr;
        let b = tokens!( @max $($exprs),+ );

        if a > b { a } else { b }
    }};
}

//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    tokens! {
        Hello: str = "Hello";
        World: str = "World";

        Boolean<str> {
            False = "false",
            True = "true",
        }
    }

    #[test]
    fn tk() {
        let pat = simple_opaque((Hello, is_ws.., World));
        assert_eq!(pat.full_match("Hello \n World").unwrap(), (Hello, " \n ", World));
    }

    #[test]
    fn tkst() {
        let pat = simple_opaque(BooleanT);
        assert_eq!(pat.full_match("true").unwrap(), Boolean::True);
        assert_eq!(pat.full_match("false").unwrap(), Boolean::False);
        assert!(pat.full_match("False").is_err());
    }
}

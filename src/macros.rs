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

#[doc(hidden)]
#[macro_export]
macro_rules! token_set {
    () => {};
}

// /// Generate structures, implement [`Precede`] for a set of tokens conveniently.
// #[macro_export]
// macro_rules! token_set {
//     ( $(
//         $(#[$attr:meta])*
//         $name:ident { $(
//             $(#[$bttr:meta])*
//             $key:ident = $word:literal
//         ),* $(,)? }
//     )* ) => { $( $crate::common::paste! {
//       $(#[$attr])*
//         #[doc = "\n\n*(generated token discriminant)*"]
//         #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//         pub(crate) enum [<$name Token>] { $(
//           $(#[$bttr])*
//             #[doc = "\n\nAssociates `` " $word " ``"]
//             $key,
//         )* }
//
//         impl [<$name Token>] {
//             /// Returns the associated text.
//             #[allow(dead_code, unreachable_patterns)]
//             pub fn text(&self) -> &'static str {
//                 match self {
//                     $( Self::$key => token_set!( @validate $word ), )*
//                     _ => unreachable!(),
//                 }
//             }
//         }
//
//         #[doc = "Generated tokens pattern with [`" [<$name Token>] "`] discriminant."
//                 "\n\nZST type by [`token_set!`] macro, only for passing as argument."]
//         pub(crate) struct [<$name Tokens>];
//
//         impl Precede for [<$name Tokens>] {
//             type Discriminant = [<$name Token>];
//
//             fn max_len(&self) -> usize {
//                 const { token_set!( @max $($word.len(),)* 0 ) }
//             }
//
//             fn matches(&self, _content: &str) -> Option<(usize, Self::Discriminant)> {
//             $(
//                 if _content.starts_with($word) {
//                     return const { Some(($word.len(), Self::Discriminant::$key)) }
//                 }
//             )*
//                 None
//             }
//         }
//     } )* };
//
//     ( @max $expr:expr ) => { $expr };
//
//     ( @max $expr:expr, $( $exprs:expr ),+ ) => {{
//         let a = $expr;
//         let b = token_set!( @max $($exprs),+ );
//
//         if a > b { a } else { b }
//     }};
//
//     ( @validate $word:literal ) => {
//         const {
//             let word = $word;
//             assert!(
//                 !word.is_empty() && word.len() <= 8192,
//                 "the associated text must be non-empty string, and no more than 8192 bytes"
//             );
//             word
//         }
//     };
// }

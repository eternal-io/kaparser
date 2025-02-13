#[doc(hidden)]
#[macro_export]
macro_rules! repeat {
    ($n:tt, $p:expr) => {
        $crate::combine::repeat::repeat_exact::<_, _, $n>($p)
    };
    ($n:tt..=$x:tt, $p:expr) => {
        $crate::combine::repeat::repeat::<_, _, $n, { $x - $n }>($p)
    };
    (..=$x:tt, $p:expr) => {
        $crate::combine::repeat::repeat_at_most::<_, _, $x>($p)
    };

    ($n:tt..$x:tt, $p:expr) => {
        compile_error!("use `n..=x` instead")
    };
    (..$x:tt, $p:expr) => {
        compile_error!("use `..=x` instead")
    };

    ($n:tt.., $p:expr) => {
        todo!()
        compile_error!("consider use `silent(...)` or `collect(...)` instead")
    };
}

//------------------------------------------------------------------------------

#[doc(hidden)]
#[macro_export]
macro_rules! token_set {
    () => {};
}

// /// Generate structures, implement [`Proceed`] for a set of tokens conveniently.
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
//         impl Proceed for [<$name Tokens>] {
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

# List of combinators.

## Primes

| prime | captured | description |
|:-----:|:-------- |:----------- |
| [`alt`](alt()) | `AltN<C₁, C₂, …, Cₙ>` | TODO! |
| [`seq`](seq()) | `(C₁, C₂, …, Cₙ)` | TODO! |
| [`com`](com()) | `&U` | TODO! |

## Definites

| definite | captured | description |
|:--------:|:-------- |:----------- |
| [`rep!`] | `[C; N]`, or<br />`[Option<C>; M]`, or<br />`([C; N], [Option<C>; M])` | TODO! |
| [`len!`] | `[T; N]` | TODO! |
| [`take`](take()) | `&U` | TODO! |
| [`when`&emsp13;`P..`](when())<br /><sup>*(RangeFrom)*</sup> | `&U` | Same as `take(1.., P)`. |

## Indefinites

| indefinite | captured | description |
|:----------:|:-------- |:----------- |
| [`till`&emsp13;`..P`](till())<br /><sup>*(RangeTo)*</sup> | `(&str, Option<char>)`<br />`(&[T], Option<T>)` | Keep capturing until encountered [`Predicate`].<br />If the EOF is encountered early, it will **still succeed**. |
| [`until`&emsp13;`..=P`](until())<br /><sup>*(RangeToInclusive)*</sup> | `(&U, C)` | Keep capturing until encountered [`Pattern`].<br />If the EOF is encountered early, it will **fail** with **No backtrack**. |

## Modifiers

| modifier | captured | description |
|:--------:|:-------- |:----------- |
| [`opt`](opt()) | `Option<C>` | May or may not match. |
| [`map`](map()) | `O` <sub>*where `Fn(C) -> O`*</sub> | Inline infallible conversions. |
| [`complex`](complex()) | `C₂` | TODO! |
| [`cond`](cond()) | `C` | TODO! |
| [`verify`](verify()) | `C` | TODO! |

However, there is no way to have combinator like `verify_map`, because `verify` and `map` happen at different stages.

## Basic patterns

| pattern | captured | comment |
|:-------:|:--------:|:------- |
| [`&U`](Pattern)<br /><sup>[*(Slice)*](Slice)</sup> | `&U` | Only implemented on primitives. |
| [`[P; 1]`](Pattern)<br /><sup>*(Array)*</sup> | `T` | [`Pattern`] by promotes single [`Predicate`]. |
| [`token_set!`] | *Variant* | `macro_rules!` TODO! |

## Basic predicates

| predicate | comment |
|:---------:|:------- |
| [`T`](Predicate) | Only implemented on primitives. |
| [`not`](not()) | - |
| [`..`](Predicate)<br /><sup>*(RangeFull)*</sup> | Asserts to `true` for any type. |
| [`T₁..=T₂`](Predicate)<br /><sup>*(RangeInclusive)*</sup> | Accepts other range bounds. |
| [`()`](Predicate)<br /><sup>*(Unit)*</sup> | Asserts to `false` for any type. |
| [`(P₁, P₂, …, Pₙ)`](Predicate)<br /><sup>*(Tuple)*</sup> | Only implemented on tuples with arity *16* or *less*. |

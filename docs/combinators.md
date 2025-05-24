<!-- # List of combinators.

*(Use `C` to denotes `Pattern::Captured`)*

## Primes

| prime | captured | description |
|:-----:|:-------- |:----------- |
| [`alt`](alt()) | `AltN<C₁, C₂, …, Cₙ>` | TODO! |
| [`seq`](Pattern) | `(C₁, C₂, …, Cₙ)` | TODO! |
| [`com`](com()) | `&U` | TODO! |

## Definites

| definite | captured | description |
|:--------:|:-------- |:----------- |
| [`rep!`] | `[C; N]`, or<br />`[Option<C>; M]`, or<br />`([C; N], [Option<C>; M])` | TODO! |
| [`len!`] | `[T; N]` | TODO! |
| [`take`](take()) | `&U` | TODO! |
| [`take0`](take0()) | `&U` | TODO! |
| [`take1`&emsp13;`P..`](take1())<br /><sup>*(RangeFrom)*</sup> | `&U` | TODO! |

## Indefinites

| indefinite | captured | description |
|:----------:|:-------- |:----------- |
| [`till`&emsp13;`..P`](till())<br /><sup>*(RangeTo)*</sup> | `(&str, Option<char>)`<br />`(&[T], Option<T>)` | Keep capturing until encountered [`Predicate`].<br />If the EOF is encountered early, it will **still succeed**. |
| [`until`&emsp13;`..=P`](until())<br /><sup>*(RangeToInclusive)*</sup> | `(&U, C)` | Keep capturing until encountered [`Pattern`].<br />If the EOF is encountered early, it will **fail** with **No backtrack**. |

## Basic patterns

| pattern | captured | comment |
|:-------:|:--------:|:------- |
| [`&U`](Pattern)<br /><sup>[*(Slice)*](Slice)</sup> | `&U` | Only implemented on primitives. |
| [`[P; 1]`](Pattern)<br /><sup>*(Array)*</sup> | `T` | [`Pattern`] by promotes single [`Predicate`]. |
| [`tokens!`] | *Variant* | `macro_rules!` TODO! |
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

## Modifiers

- [`cut`](cut()):
- [`opt`](opt()):
- [`cond`](cond()):
- [`map`](Pattern::map()):
- [`map_err`](Pattern::map_err()):
- [`desc`](Pattern::desc()):
- [`desc_with`](Pattern::desc_with()):
- [`unwrap`](Pattern::unwrap()):
- [`or`](Pattern::or()):
- [`or_else`](Pattern::or_else()):
- [`or_default`](Pattern::or_default()):
- [`complex`](Pattern::complex()): -->

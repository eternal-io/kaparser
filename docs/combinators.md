List of combinators.

## Primes

- `alt`
- `com`
- `seq`

## Definites

- `rep!`
- `len!`
- `take`
- `when` / `P..` <sub>(*RangeFrom*, abbreviation of `take(1.., _)`)</sub>

## Indefinites

| Indefinite | Captured | Description |
|:----------:|:-------- |:----------- |
| [`till`&emsp13;`..P`](till())<br /><sup>*(RangeTo)*</sup> | `(&str, Option<char>)`<br />`(&[T], Option<T>)` | Keep capturing until encountered [`Predicate`].<br />If the EOF is encountered early, it will **still succeed**. |
| [`until`&emsp13;`..=P`](until())<br /><sup>*(RangeToInclusive)*</sup> | `(&U, P::Captured)` | Keep capturing until encountered [`Pattern`].<br />If the EOF is encountered early, it will **fail** with **No backtrack**. |

## Modifiers

| Modifier | Captured | Description |
|:--------:|:-------- |:----------- |
| [`opt`](opt()) | `Option<P::Captured>` | May or may not match. |
| [`map`](map()) | `O` <sub>*where `Fn(P::Captured) -> O`*</sub> | Inline infallible conversions. |
| [`complex`](complex()) | `Q::Captured` | TODO! |
| [`cond`](cond()) | `P::Captured` | TODO! |
| [`verify`](verify()) | `P::Captured` | TODO! |

However, there is no way to have combinator like `verify_map`, because these two happen at different stages.

## Basic patterns

| [Pattern] | Captured | Example | Result | Note |
|:---------:|:--------:|:------- |:------ |:---- |
| [`&U`](Pattern)<br /><sup>[*(Slice)*](Slice)</sup> | `&U` | `"token".full_match("token")` | `Ok("token")` | Only implemented on primitives. |
| [`[P; 1]`](Pattern)<br /><sup>*(Array)*</sup> | `T` | `[is_hex].full_match("E")` | `Ok('E')` | A [`Pattern`] by promotes a single [`Predicate`]. |
| [`token_set!`] | *Variant* | - | - | `macro_rules!` TODO! |

- [`Predicate<char>`] promotes to [`Pattern<str>`], and
- [`Predicate<T>`] <sub>where `T: Copy + PartialEq`</sub> promotes to [`Pattern<[T]>`](Pattern).

## Basic predicates

| [Predicate] | Example | Result | Note |
|:-----------:|:------- |:------ |:---- |
| [`T`](Predicate) | `'I'.predicate('I')` | `true` | Only implemented on primitives. |
| [`..`](Predicate)<br /><sup>*(RangeFull)*</sup>  | `(..).predicate(0)`<br />`(..).predicate(255)` | `true`<br />`true` | Asserts to `true` for any type. |
| [`T₁..=T₂`](Predicate)<br /><sup>*(RangeInclusive)*</sup> | `('A'..='B').predicate('B')`<br />`('A'..='B').predicate('C')` | `true`<br />`false` | Accepts other range bounds. |
| [`()`](Predicate)<br /><sup>*(Unit)*</sup> | `(..).predicate(0)`<br />`(..).predicate(255)` | `false`<br />`false` | Asserts to `false` for any type. |
| [`(P₁, P₂, …, Pₙ)`](Predicate)<br /><sup>*(Tuple)*</sup> | `('X', 'Y').predicate('Y')`<br />`('X', 'Y').predicate('Z')` | `true`<br />`false` | Only implemented on tuples with arity *16* or *less*. |
| [`not`](not()) | `not(is_hex).predicate('E')`<br />`not(is_hex).predicate('g')` | `false`<br />`true` | - |

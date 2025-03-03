#![allow(clippy::type_complexity)]
#![allow(non_snake_case)]
use crate::anything::*;

pub const fn LINE_END<'a, E: Situation>() -> Compound<&'a str, E, (Optional<&'a str, E, &'a str>, &'a str)> {
    com((opt("\r"), "\n"))
}

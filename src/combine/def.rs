#![allow(clippy::type_complexity)]
#![allow(non_snake_case)]
use crate::anything::*;

pub const fn LINE_END<E: Situation>()
-> Compound<&'static str, E, (Optional<&'static str, E, &'static str>, &'static str)> {
    com((opt("\r"), "\n"))
}

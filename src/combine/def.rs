#![allow(clippy::type_complexity)]
use crate::anything::*;

pub const LINE_END: Compound<&str, (Optional<&str, &str>, &str)> = com((opt("\r"), "\n"));

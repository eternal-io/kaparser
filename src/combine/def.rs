use crate::anything::*;

pub const LINE_END: Compound<'_, str, (Optional<'_, str, &str>, &str)> = com((opt("\r"), "\n"));

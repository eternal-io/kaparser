use super::*;

pub const CRLF: Compound<'_, str, (Optional<'_, str, &str>, &str)> = com((opt("\r"), "\n"));

use kaparser::prelude::*;

#[test]
fn times() {}

#[test]
fn one_more() {
    assert_eq!({ is_dec.. }.full_match("!").unwrap_err(), 0);
    assert_eq!({ is_dec.. }.full_match("0123!").unwrap_err(), 4);
    assert_eq!({ is_dec.. }.full_match("7890").unwrap(), "7890");
    assert_eq!({ is_dec.. }.parse_next(&mut "!").unwrap_err(), 0);
    assert_eq!({ is_dec.. }.parse_next(&mut "0123!").unwrap(), "0123");
    assert_eq!({ is_dec.. }.parse_next(&mut "7890").unwrap(), "7890");

    assert_eq!({ not(0).. }.full_match(b"\0").unwrap_err(), 0);
    assert_eq!({ not(0).. }.full_match(b"0123\0").unwrap_err(), 4);
    assert_eq!({ not(0).. }.full_match(b"7890").unwrap(), b"7890");
    assert_eq!({ not(0).. }.parse_next(&mut b"\0".as_slice()).unwrap_err(), 0);
    assert_eq!({ not(0).. }.parse_next(&mut b"0123\0".as_slice()).unwrap(), b"0123");
    assert_eq!({ not(0).. }.parse_next(&mut b"7890".as_slice()).unwrap(), b"7890");
}

#[test]
#[cfg(feature = "std")]
fn streaming() {
    let s = "efab6251-2b3e-4395-BFC0-370E268935D1";
    let mut par = Parser::from_reader_in_str_with_capacity(s.as_bytes(), 0);

    assert_eq!(par.proceed_str((is_hex, '-')..).unwrap(), s);
    assert!(par.exhausted());
}

use kaparser::prelude::*;

#[test]
fn times() {
    assert_eq!(take(1..3, unc::upper).full_match("").unwrap_err(), 0);
    assert_eq!(take(1..3, unc::upper).full_match("Ａ").unwrap(), "Ａ");
    assert_eq!(take(1..3, unc::upper).full_match("ＡＢ").unwrap(), "ＡＢ");
    assert_eq!(take(1..3, unc::upper).full_match("ＡＢＣ").unwrap_err(), 6);

    assert_eq!(take(2..=3, is_alpha).full_match("").unwrap_err(), 0);
    assert_eq!(take(2..=3, is_alpha).full_match("a").unwrap_err(), 1);
    assert_eq!(take(2..=3, is_alpha).full_match("ab").unwrap(), "ab");
    assert_eq!(take(2..=3, is_alpha).full_match("abc").unwrap(), "abc");
    assert_eq!(take(2..=3, is_alpha).full_match("abcd").unwrap_err(), 3);

    assert_eq!(take(4, is_alpha).full_match("abc").unwrap_err(), 3);
    assert_eq!(take(4, is_alpha).full_match("abcd").unwrap(), "abcd");
    assert_eq!(take(4, is_alpha).full_match("abcde").unwrap_err(), 4);

    assert_eq!(take(4, not(0)).full_match(b"abc\0").unwrap_err(), 3);
    assert_eq!(take(4, not(0)).full_match(b"abc\n").unwrap(), b"abc\n");

    assert_eq!(take(2..=3, not(0)).parse_next(&mut b"a\0".as_ref()).unwrap_err(), 1);
    assert_eq!(take(2..=3, not(0)).parse_next(&mut b"ab\0d".as_ref()).unwrap(), b"ab");
    assert_eq!(take(2..=3, not(0)).parse_next(&mut b"ab\nd".as_ref()).unwrap(), b"ab\n");
}

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
    assert_eq!({ not(0).. }.parse_next(&mut b"\0".as_ref()).unwrap_err(), 0);
    assert_eq!({ not(0).. }.parse_next(&mut b"0123\0".as_ref()).unwrap(), b"0123");
    assert_eq!({ not(0).. }.parse_next(&mut b"7890".as_ref()).unwrap(), b"7890");
}

#[test]
#[cfg(feature = "std")]
fn streaming() {
    let s = "EFAB6251-2b3e-4395-bfc0-370e268935d1";
    let pat = seq((
        take(8, is_hex),
        "-",
        take(4, is_hex),
        "-",
        take(4, is_hex),
        "-",
        take(4, is_hex),
        "-",
        is_hex..,
    ));

    let mut par = Parser::from_reader_in_str_with_capacity(s.as_bytes(), 0);

    assert_eq!(
        par.proceed_str(pat).unwrap(),
        ("EFAB6251", "-", "2b3e", "-", "4395", "-", "bfc0", "-", "370e268935d1")
    );

    assert!(par.exhausted());
}

use super::*;

const FULL_WIDTH_XYZ: &[char] = &['Ｘ', 'Ｙ', 'Ｚ'];

#[test]
fn takes() -> TheResult<()> {
    let mut rdr = Utf8Parser::from_str("Ｘ01234567Ｙ89abcdefＺ");

    assert_eq!(rdr.take_once(FULL_WIDTH_XYZ)?, Some(FULL_WIDTH_XYZ[0]));
    assert_eq!(rdr.take_times(is_digit, 2)?, Ok(("01", Some('2'))));
    assert_eq!(
        rdr.take_while(all!(is_octdigit, FULL_WIDTH_XYZ))?,
        ("234567Ｙ", Some('8'))
    );

    assert_eq!(rdr.take_times(not!(is_alphabetic), 3..)?, (Err(("89", Some('a')))));

    assert_eq!(rdr.take_while(any)?, ("89abcdefＺ", None));

    assert!(rdr.exhausted());

    Ok(())
}

token_set! {
    Word {
        Foo = "Foo",
        Bar = "Bar",
        Baz = "Baz",
        Hello = "你好",
    }
}

#[test]
fn matches() -> TheResult<()> {
    assert_eq!(Utf8Parser::from_str("Baz").matches(WordTokens)?, Some(WordToken::Baz));
    assert_eq!(
        Utf8Parser::from_str("你好").matches(WordTokens)?,
        Some(WordToken::Hello)
    );
    assert_eq!(Utf8Parser::from_str("Boom").matches(WordTokens)?, None);

    Ok(())
}

#[test]
fn until() -> TheResult<()> {
    let mut rdr = Utf8Parser::from_str("~Boom~Baz~Bar~Foo~你好~");

    assert_eq!(rdr.skim_until(WordTokens)?, Ok(("~Boom~", WordToken::Baz)));
    assert_eq!(rdr.skim_until(WordTokens)?, Ok(("~", WordToken::Bar)));
    assert_eq!(rdr.skim_until(WordTokens)?, Ok(("~", WordToken::Foo)));
    assert_eq!(rdr.skim_until(WordTokens)?, Ok(("~", WordToken::Hello)));
    assert_eq!(rdr.skim_until(WordTokens)?, Err("~"));

    Ok(())
}

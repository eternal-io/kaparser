use super::*;

const FULL_WIDTH_XYZ: &[char] = &['Ｘ', 'Ｙ', 'Ｚ'];

#[test]
fn takes() -> TheResult<()> {
    let mut rdr = Utf8Parser::from_str("Ｘ01234567Ｙ89abcdefＺ");

    assert_eq!(rdr.take_once(FULL_WIDTH_XYZ)?, Some(FULL_WIDTH_XYZ[0]));
    assert_eq!(rdr.take_times(is_digit, 2)?, Some(("01", Some('2'))));
    assert_eq!(rdr.take_while((is_octdigit, FULL_WIDTH_XYZ))?, ("234567Ｙ", Some('8')));

    assert_eq!(rdr.take_times(not!(is_alphabetic), 3..)?, None);

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

    assert_eq!(rdr.skim_until(WordTokens)?, Some(("~Boom~", WordToken::Baz)));
    assert_eq!(rdr.skim_until(WordTokens)?, Some(("~", WordToken::Bar)));
    assert_eq!(rdr.skim_until(WordTokens)?, Some(("~", WordToken::Foo)));
    assert_eq!(rdr.skim_until(WordTokens)?, Some(("~", WordToken::Hello)));
    assert_eq!(rdr.skip_until(WordTokens)?, ("~", None));

    Ok(())
}

#[test]
fn select() -> TheResult<()> {
    let mut rdr = Utf8Parser::from_str("-123.456");

    rdr.begin_select();

    assert_eq!(rdr.take_once('-')?, Some('-'));
    assert_eq!(rdr.take_while(is_digit)?, ("123", Some('.')));
    assert_eq!(rdr.next()?, Some('.'));

    assert_eq!(rdr.selection(), Some("-123."));
    rdr.bump(3);
    assert!(rdr.exhausted());
    assert!(rdr.content().is_empty());

    assert_eq!(rdr.rollback_select(), Some("-123.456"));
    assert_eq!(rdr.content(), "-123.456");

    Ok(())
}

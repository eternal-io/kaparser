use kaparser::prelude::*;

#[derive(Debug, PartialEq, Eq)]
enum Color {
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, u8),
}

fn parse_color(s: &str) -> Option<Color> {
    // This is really a constant! But you don't want to write out its type...
    let pat = const { seq(("#", rep!(3..=4, take(2, is_hex)))) };
    let (_t, ([r, g, b], [a])) = pat.matches(s)?;
    let r = u8::from_str_radix(r, 16).unwrap();
    let g = u8::from_str_radix(g, 16).unwrap();
    let b = u8::from_str_radix(b, 16).unwrap();
    match a {
        None => Some(Color::Rgb(r, g, b)),
        Some(a) => Some(Color::Rgba(r, g, b, u8::from_str_radix(a, 16).unwrap())),
    }
}

fn main() {
    assert_eq!(parse_color("#1123EE").unwrap(), Color::Rgb(0x11, 0x23, 0xEE));
    assert_eq!(parse_color("#69F0AE").unwrap(), Color::Rgb(0x69, 0xF0, 0xAE));
    assert_eq!(parse_color("#ffab00ff").unwrap(), Color::Rgba(0xff, 0xab, 0, 0xff));
    assert!(parse_color("#fabcxx").is_none());
}

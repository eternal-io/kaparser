use crate::prelude::*;
use core::fmt::Debug;

pub use std::prelude::rust_2024::*;

pub type FullMatchSamples<Cap> = Vec<(&'static str, Result<Cap, usize>)>;
pub type PartialMatchSamples<Cap> = Vec<(&'static str, Result<(Cap, &'static str), usize>)>;

pub fn test_full_match<'i, Cap: Debug + PartialEq>(
    pattern: impl Pattern<'i, str, SimpleError, Captured = Cap>,
    samples: FullMatchSamples<Cap>,
) {
    let mut ok_flag = true;

    for (slice, expected) in samples.into_iter() {
        match pattern.full_match(slice) {
            Ok(cap) => match expected {
                Ok(cap_exp) => match cap == cap_exp {
                    true => continue,

                    false => eprintln!("expected Ok({:?}), found Ok({:?})", cap_exp, cap),
                },

                Err(_) => eprintln!("expected Err(_), found Ok({:?})", cap),
            },

            Err(e) => match expected {
                Err(off_exp) => match e.offset() == off_exp {
                    true => continue,

                    false => eprintln!("expected Err({}), found Err({})", off_exp, e.offset()),
                },

                Ok(_) => eprintln!("expected Ok(_), found Err({:?})", e.offset()),
            },
        }

        ok_flag = false;
    }

    assert!(ok_flag);
}

pub fn test_partial_match<'i, Cap: Debug + PartialEq>(
    pattern: impl Pattern<'i, str, SimpleError, Captured = Cap>,
    samples: PartialMatchSamples<Cap>,
) {
    let mut ok_flag = true;

    for (mut slice, expected) in samples.into_iter() {
        let rest = &mut slice;
        match pattern.parse(rest) {
            Ok(cap) => match expected {
                Ok((cap_exp, rest_exp)) => match (cap == cap_exp, *rest == rest_exp) {
                    (true, true) => continue,

                    (cap_eq, rest_eq) => {
                        eprintln!(
                            "expected Ok({}, {}),\n   found Ok({}, {})",
                            match cap_eq {
                                true => format!("_"),
                                false => format!("{cap_exp:?}"),
                            },
                            match rest_eq {
                                true => format!("_"),
                                false => format!("{rest_exp:?}"),
                            },
                            match cap_eq {
                                true => format!("_"),
                                false => format!("{cap:?}"),
                            },
                            match rest_eq {
                                true => format!("_"),
                                false => format!("{rest:?}"),
                            },
                        );
                    }
                },

                Err(_) => eprintln!("expected Err(_), found Ok({:?})", cap),
            },

            Err(e) => match expected {
                Err(off_exp) => match e.offset() == off_exp {
                    true => continue,

                    false => eprintln!("expected Err({}), found Err({})", off_exp, e.offset()),
                },

                Ok(_) => eprintln!("expected Ok(_), found Err({:?})", e.offset()),
            },
        }

        ok_flag = false;
    }

    assert!(ok_flag);
}

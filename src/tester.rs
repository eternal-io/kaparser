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

    for (i, (slice, expected)) in samples.into_iter().enumerate() {
        match pattern.fullmatch(slice) {
            Ok(cap) => match expected {
                Ok(cap_exp) => match cap == cap_exp {
                    true => continue,

                    false => eprintln!("#{i} expected Ok({:?}), found Ok({:?})", cap_exp, cap),
                },

                Err(off_exp) => eprintln!("#{i} expected Err({}), found Ok({:?})", off_exp, cap),
            },

            Err(e) => match expected {
                Err(off_exp) => match e.offset() == off_exp {
                    true => continue,

                    false => eprintln!(
                        "#{i} expected Err({}), found Err({}) ({})",
                        off_exp,
                        e.offset(),
                        match e.is_rejected() {
                            true => "reject",
                            false => "HALT",
                        }
                    ),
                },

                Ok(cap_exp) => eprintln!(
                    "#{i} expected Ok({:?}), found Err({}) ({})",
                    cap_exp,
                    e.offset(),
                    match e.is_rejected() {
                        true => "reject",
                        false => "HALT",
                    }
                ),
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

    for (i, (mut slice, expected)) in samples.into_iter().enumerate() {
        let rest = &mut slice;
        match pattern.parse(rest) {
            Ok(cap) => match expected {
                Ok((cap_exp, rest_exp)) => match (cap == cap_exp, *rest == rest_exp) {
                    (true, true) => continue,

                    (cap_eq, rest_eq) => {
                        eprintln!(
                            "#{i} expected Ok({}, {}),\n       found Ok({}, {})",
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

                Err(off_exp) => eprintln!("#{i} expected Err({}), found Ok({:?})", off_exp, cap),
            },

            Err(e) => match expected {
                Err(off_exp) => match e.offset() == off_exp {
                    true => continue,

                    false => eprintln!(
                        "#{i} expected Err({}), found Err({}) ({})",
                        off_exp,
                        e.offset(),
                        match e.is_rejected() {
                            true => "reject",
                            false => "HALT",
                        }
                    ),
                },

                Ok(cap_exp) => eprintln!(
                    "#{i} expected Ok({:?}), found Err({}) ({})",
                    cap_exp,
                    e.offset(),
                    match e.is_rejected() {
                        true => "reject",
                        false => "HALT",
                    }
                ),
            },
        }

        ok_flag = false;
    }

    assert!(ok_flag);
}

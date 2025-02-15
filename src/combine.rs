use crate::{common::*, predicate::*, precede::*};
use core::{marker::PhantomData, mem::MaybeUninit};

pub mod alternate;
pub mod compound;
pub mod cut;
pub mod fasten;
pub mod joined;
pub mod not;
pub mod reiterate;
pub mod repeat;
pub mod sequence;
pub mod skim;
pub mod take;
pub mod winged;

use crate::prelude::*;

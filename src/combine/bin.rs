#![allow(clippy::type_complexity)]
use crate::anything::*;

//------------------------------------------------------------------------------

// pub const LE_U32: Map<'_, [u8], Lens<'_, u8, RangeFull, 4>, fn([u8; 4]) -> u32, u32> =
//     map(u32::from_le_bytes, len!(4, ..));
// pub const LE_U64: Map<'_, [u8], Lens<'_, u8, RangeFull, 8>, fn([u8; 8]) -> u64, u64> =
//     map(u64::from_le_bytes, len!(8, ..));

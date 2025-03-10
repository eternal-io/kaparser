use crate::anything::*;

macro_rules! gen_binary_patterns {
    ( $(
      $(#[$attr:meta])*
        $f:ident, $ty:ty, $len:tt;
    )* ) => { paste::paste! { $(
      $(#[$attr])*
        #[inline(always)]
        pub const fn [<$f _ $ty>]<'i, E: Situation>()
            -> Map<'i, [u8], E, Lens<u8, RangeFull, E, $len>, fn([u8; $len]) -> $ty, $ty>
        {
            map($ty::[<from_ $f _bytes>], len!($len, ..))
        }
    )* } };
}

gen_binary_patterns! {
    /** little-endian       signed  1 bytes integer */ le,   i8,  1;
    /** little-endian       signed  2 bytes integer */ le,  i16,  2;
    /** little-endian       signed  4 bytes integer */ le,  i32,  4;
    /** little-endian       signed  8 bytes integer */ le,  i64,  8;
    /** little-endian       signed 16 bytes integer */ le, i128, 16;
    /** little-endian     unsigned  1 bytes integer */ le,   u8,  1;
    /** little-endian     unsigned  2 bytes integer */ le,  u16,  2;
    /** little-endian     unsigned  4 bytes integer */ le,  u32,  4;
    /** little-endian     unsigned  8 bytes integer */ le,  u64,  8;
    /** little-endian     unsigned 16 bytes integer */ le, u128, 16;
    /** little-endian 4 bytes floating point number */ le,  f32,  4;
    /** little-endian 8 bytes floating point number */ le,  f64,  8;

    /**    big-endian       signed  1 bytes integer */ be,   i8,  1;
    /**    big-endian       signed  2 bytes integer */ be,  i16,  2;
    /**    big-endian       signed  4 bytes integer */ be,  i32,  4;
    /**    big-endian       signed  8 bytes integer */ be,  i64,  8;
    /**    big-endian       signed 16 bytes integer */ be, i128, 16;
    /**    big-endian     unsigned  1 bytes integer */ be,   u8,  1;
    /**    big-endian     unsigned  2 bytes integer */ be,  u16,  2;
    /**    big-endian     unsigned  4 bytes integer */ be,  u32,  4;
    /**    big-endian     unsigned  8 bytes integer */ be,  u64,  8;
    /**    big-endian     unsigned 16 bytes integer */ be, u128, 16;
    /**    big-endian 4 bytes floating point number */ be,  f32,  4;
    /**    big-endian 8 bytes floating point number */ be,  f64,  8;

    /** native-endian       signed  1 bytes integer */ ne,   i8,  1;
    /** native-endian       signed  2 bytes integer */ ne,  i16,  2;
    /** native-endian       signed  4 bytes integer */ ne,  i32,  4;
    /** native-endian       signed  8 bytes integer */ ne,  i64,  8;
    /** native-endian       signed 16 bytes integer */ ne, i128, 16;
    /** native-endian     unsigned  1 bytes integer */ ne,   u8,  1;
    /** native-endian     unsigned  2 bytes integer */ ne,  u16,  2;
    /** native-endian     unsigned  4 bytes integer */ ne,  u32,  4;
    /** native-endian     unsigned  8 bytes integer */ ne,  u64,  8;
    /** native-endian     unsigned 16 bytes integer */ ne, u128, 16;
    /** native-endian 4 bytes floating point number */ ne,  f32,  4;
    /** native-endian 8 bytes floating point number */ ne,  f64,  8;
}

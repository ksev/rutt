use std::u128;

pub trait Zero {
    const ZERO: Self;
}

macro_rules! impl_zero {
    ($const: literal, $type:ty) => {
        impl Zero for $type {
            const ZERO: $type = $const;
        }
    }
}

impl_zero!(0, usize);
impl_zero!(0, isize);
impl_zero!(0, i8);
impl_zero!(0, u8);
impl_zero!(0, i16);
impl_zero!(0, u16);
impl_zero!(0, i32);
impl_zero!(0, u32);
impl_zero!(0, i64);
impl_zero!(0, u64);
impl_zero!(0, u128);
impl_zero!(0, i128);

impl_zero!(0.0, f32);
impl_zero!(0.0, f64);

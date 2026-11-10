use core::ops::{Add, Mul};

pub trait Number: Copy + PartialOrd + Add<Output = Self> + Mul<Output = Self> {
    const ZERO: Self;
}
macro_rules! impl_number { ($($t:ty),*) => { $(impl Number for $t { const ZERO: Self = 0 as $t; })* } }
impl_number!(u8, u16, u32, u64, i32, i64, f32, f64);

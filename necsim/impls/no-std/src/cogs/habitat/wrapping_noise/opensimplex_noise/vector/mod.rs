pub mod vec2;
pub mod vec3;
pub mod vec4;

use core::ops::{Add, Mul, Sub};

pub trait VecMethods<T> {
    fn sum(&self) -> T;
    fn get_attenuation_factor(&self) -> T;
}

pub trait VecType<T>:
    VecMethods<T>
    + Copy
    + Sub<Self, Output = Self>
    + Add<Self, Output = Self>
    + Mul<T, Output = Self>
    + core::marker::Sized
{
}
impl<
        T,
        X: VecMethods<T>
            + Copy
            + Sub<Self, Output = Self>
            + Add<Self, Output = Self>
            + Mul<T, Output = Self>,
    > VecType<T> for X
{
}

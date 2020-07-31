//! 3-D normals

#![allow(dead_code)]
use super::common::*;
use super::{Axis, Float, Vector3};
use num_traits::{Num, Zero};
use std::ops;

/// A 3-D normal containing numeric values.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Normal3<T> {
    /// X-coordinate.
    pub x: T,

    /// Y-coordinate.
    pub y: T,

    /// Y-coordinate.
    pub z: T,
}

/// 3-D normal containing `Float` values.
pub type Normal3f = Normal3<Float>;

/// Creates a new 3-D normal.
///
/// * `x`: X-coordinate.
/// * `y`: Y-coordinate.
/// * `z`: Z-coordinate.
pub fn normal3<T>(x: T, y: T, z: T) -> Normal3<T> {
    Normal3 { x, y, z }
}

/// Creates a new 3-D zero normal.
pub fn zero_normal3<T: Zero>() -> Normal3<T> {
    normal3(T::zero(), T::zero(), T::zero())
}

impl<T: Num> Normal3<T> {
    /// Returns true if either coordinate is NaN.
    pub fn has_nans(&self) -> bool
    where
        T: num_traits::Float,
    {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }

    /// Returns the square of the normal's length.
    pub fn length_squared(&self) -> T
    where
        T: ops::Mul<Output = T> + ops::Add<Output = T> + Copy,
    {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the normal's length.
    pub fn length(&self) -> T
    where
        T: num_traits::Float,
    {
        self.length_squared().sqrt()
    }

    /// Returns the unit normal.
    pub fn normalize(&self) -> Self
    where
        T: num_traits::Float,
    {
        *self / self.length()
    }
}

impl<T: Num + ops::Neg<Output = T> + PartialOrd + Copy> Dot<Normal3<T>> for Normal3<T> {
    type Output = T;

    /// Returns the dot product with another normal.
    ///
    /// * `other` - The other normal.
    fn dot(&self, other: &Normal3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

impl<T: Num + ops::Neg<Output = T> + PartialOrd + Copy> Dot<Vector3<T>> for Normal3<T> {
    type Output = T;

    /// Returns the dot product with another vector.
    ///
    /// * `other` - The other vector.
    fn dot(&self, other: &Vector3<T>) -> T {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

/// Implement FaceForward trait which allows pointing vectors in the same
/// hemisphere as another normal/vector.
impl<T: Num + ops::Neg<Output = T> + PartialOrd + Copy> FaceForward<T, Vector3<T>> for Normal3<T> {}

impl<T: Num> ops::Add for Normal3<T> {
    type Output = Normal3<T>;

    /// Adds the given normal and returns the result.
    ///
    /// * `other` - The normal to add.
    fn add(self, other: Self) -> Self::Output {
        normal3(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: Num + Copy> ops::AddAssign for Normal3<T> {
    /// Performs the `+=` operation.
    ///
    /// * `other` - The normal to add.
    fn add_assign(&mut self, other: Self) {
        *self = normal3(self.x + other.x, self.y + other.y, self.z + other.z);
    }
}

impl<T: Num> ops::Sub for Normal3<T> {
    type Output = Normal3<T>;

    /// Subtracts the given normal and returns the result.
    ///
    /// * `other` - The normal to subtract.
    fn sub(self, other: Self) -> Self::Output {
        normal3(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl<T: Num + Copy> ops::SubAssign for Normal3<T> {
    /// Performs the `-=` operation.
    ///
    /// * `other` - The normal to subtract.
    fn sub_assign(&mut self, other: Self) {
        *self = normal3(self.x - other.x, self.y - other.y, self.z - other.z);
    }
}

impl<T: Num + Copy> ops::Mul<T> for Normal3<T> {
    type Output = Normal3<T>;

    /// Scale the vector.
    ///
    /// * `f` - The scaling factor.
    fn mul(self, f: T) -> Self::Output {
        normal3(f * self.x, f * self.y, f * self.z)
    }
}

macro_rules! premul {
    ($t: ty) => {
        impl ops::Mul<Normal3<$t>> for $t {
            type Output = Normal3<$t>;
            /// Scale the normal.
            ///
            /// * `n` - The normal.
            fn mul(self, n: Normal3<$t>) -> Normal3<$t> {
                normal3(self * n.x, self * n.y, self * n.z)
            }
        }
    };
}

premul!(f32);
premul!(f64);
premul!(i8);
premul!(i16);
premul!(i32);
premul!(i64);
premul!(u8);
premul!(u16);
premul!(u32);
premul!(u64);

impl<T: Num + Copy> ops::MulAssign<T> for Normal3<T> {
    /// Scale and assign the result to the vector.
    ///
    /// * `f` - The scaling factor.
    fn mul_assign(&mut self, f: T) {
        *self = normal3(f * self.x, f * self.y, f * self.z);
    }
}

impl<T: Num + Copy> ops::Div<T> for Normal3<T> {
    type Output = Normal3<T>;

    /// Scale the vector by 1/f.
    ///
    /// * `f` - The scaling factor.
    fn div(self, f: T) -> Self::Output {
        debug_assert!(!f.is_zero());

        let inv = T::one() / f;
        normal3(inv * self.x, inv * self.y, inv * self.z)
    }
}

impl<T: Num + Copy> ops::DivAssign<T> for Normal3<T> {
    /// Scale the vector by 1/f and assign the result to the vector.
    ///
    /// * `f` - The scaling factor.
    fn div_assign(&mut self, f: T) {
        debug_assert!(!f.is_zero());

        let inv = T::one() / f;
        *self = normal3(inv * self.x, inv * self.y, inv * self.z);
    }
}

impl<T: Num + ops::Neg<Output = T>> ops::Neg for Normal3<T> {
    type Output = Normal3<T>;

    /// Flip the vector's direction (scale by -1).
    fn neg(self) -> Self::Output {
        normal3(-self.x, -self.y, -self.z)
    }
}

impl<T> ops::Index<Axis> for Normal3<T> {
    type Output = T;

    /// Index the vector by an axis to get the immutable coordinate axis value.
    ///
    /// * `axis` - A 2-D coordinate axis.
    fn index(&self, axis: Axis) -> &Self::Output {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}

impl<T> ops::IndexMut<Axis> for Normal3<T> {
    /// Index the vector by an axis to get a mutable coordinate axis value.
    ///
    /// * `axis` - A 2-D coordinate axis.
    fn index_mut(&mut self, axis: Axis) -> &mut Self::Output {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
            Axis::Z => &mut self.z,
        }
    }
}

impl<T> From<Vector3<T>> for Normal3<T> {
    /// Convert a 3-D vector to a 3-D normal.
    ///
    /// * `v` - 3-D vector.
    fn from(v: Vector3<T>) -> Self {
        normal3(v.x, v.y, v.z)
    }
}

// ----------------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------------

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn zero_vector() {
        assert!(normal3(0, 0, 0) == zero_normal3());
        assert!(normal3(0.0, 0.0, 0.0) == zero_normal3());
    }

    #[test]
    fn has_nans() {
        assert!(!normal3(0.0, 0.0, 0.0).has_nans());
        assert!(normal3(f32::NAN, f32::NAN, f32::NAN).has_nans());
        assert!(normal3(f64::NAN, f64::NAN, f64::NAN).has_nans());
    }

    #[test]
    #[should_panic]
    fn normalize_zero_f64() {
        zero_normal3::<f64>().normalize();
    }

    #[test]
    #[should_panic]
    fn normalize_zero_f32() {
        zero_normal3::<f32>().normalize();
    }

    #[test]
    #[should_panic]
    #[allow(unused)]
    fn div_zero_i64() {
        zero_normal3::<i64>() / 0;
    }

    #[test]
    #[should_panic]
    #[allow(unused)]
    fn div_zero_f64() {
        normal3::<f64>(1.0, 1.0, 1.0) / 0.0;
    }

    // Define some properties for tests.
    prop_range!(range_i32, i32, -100..100i32);
    prop_range!(range_f32, f32, -100.0..100.0f32);

    prop_non_zero_range!(non_zero_i32, i32, -100..100i32);
    prop_non_zero_range!(non_zero_f32, f32, -100.0..100.0f32);

    prop_normal3!(normal3_i32, i32, -100..100i32, -100..100i32, -100..100i32);
    prop_normal3!(
        normal3_f32,
        f32,
        -100.0..100.0f32,
        -100.0..100.0f32,
        -100.0..100.0f32
    );

    proptest! {
        #[test]
        fn length_squared_i32(n in normal3_i32()) {
            prop_assert_eq!(n.length_squared(), n.x * n.x + n.y * n.y + n.z * n.z);
        }

        #[test]
        fn length_squared_f32(n in normal3_f32()) {
            prop_assert_eq!(n.length_squared(), n.x * n.x + n.y * n.y + n.z * n.z);
        }

        #[test]
        fn length_f32(n in normal3_f32()) {
            prop_assert_eq!(n.length(), (n.x * n.x + n.y * n.y + n.z * n.z).sqrt());
        }

        #[test]
        fn normalize_f32(n in normal3_f32()) {
            // Since we do 1.0 / l in implementation we have to do the
            // same here. Doing normal3(x / l, y / l) will not work
            // for some of the floating point values due to precision
            // errors.
            let f = 1.0 / (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            prop_assert_eq!(n.normalize(), normal3(n.x * f, n.y * f, n.z * f));
        }

        #[test]
        fn dot_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            prop_assert_eq!(n1.dot(&n2), n1.x * n2.x + n1.y * n2.y + n1.z * n2.z);
        }

        #[test]
        fn abs_dot_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            prop_assert_eq!(n1.abs_dot(&n2), (n1.x * n2.x + n1.y * n2.y + n1.z * n2.z).abs());
        }

        #[test]
        fn add_i32(n1 in normal3_i32(), n2 in normal3_i32()) {
            prop_assert_eq!(n1 + n2, normal3(n1.x + n2.x, n1.y + n2.y, n1.z + n2.z));
        }

        #[test]
        fn add_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            prop_assert_eq!(n1 + n2, normal3(n1.x + n2.x, n1.y + n2.y, n1.z + n2.z));
        }

        #[test]
        fn add_assign_i32(n1 in normal3_i32(), n2 in normal3_i32()) {
            let mut n = n1;
            n += n2;
            prop_assert_eq!(n, normal3(n1.x + n2.x, n1.y + n2.y, n1.z + n2.z));
        }

        #[test]
        fn add_assign_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            let mut n = n1;
            n += n2;
            prop_assert_eq!(n, normal3(n1.x + n2.x, n1.y + n2.y, n1.z + n2.z));
        }

        #[test]
        fn sub_i32(n1 in normal3_i32(), n2 in normal3_i32()) {
            prop_assert_eq!(n1 - n2, normal3(n1.x - n2.x, n1.y - n2.y, n1.z - n2.z));
        }

        #[test]
        fn sub_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            prop_assert_eq!(n1 - n2, normal3(n1.x - n2.x, n1.y - n2.y, n1.z - n2.z));
        }

        #[test]
        fn sub_assign_i32(n1 in normal3_i32(), n2 in normal3_i32()) {
            let mut n = n1;
            n -= n2;
            prop_assert_eq!(n, normal3(n1.x - n2.x, n1.y - n2.y, n1.z - n2.z));
        }

        #[test]
        fn sub_assign_f32(n1 in normal3_f32(), n2 in normal3_f32()) {
            let mut n = n1;
            n -= n2;
            prop_assert_eq!(n, normal3(n1.x - n2.x, n1.y - n2.y, n1.z - n2.z));
        }

        #[test]
        fn mul_i32(n in normal3_i32(), f in range_i32()) {
            let expected = normal3(n.x * f, n.y * f, n.z * f);
            prop_assert_eq!(n * f, expected);
            prop_assert_eq!(f * n, expected);
        }

        #[test]
        fn mul_f32(n in normal3_f32(), f in range_f32()) {
            let expected = normal3(n.x * f, n.y * f, n.z * f);
            prop_assert_eq!(n * f, expected);
            prop_assert_eq!(f * n, expected);
        }

        #[test]
        fn mul_assign_i32(n in normal3_i32(), f in range_i32()) {
            let mut n1 = n;
            n1 *= f;
            prop_assert_eq!(n1, normal3(n.x * f, n.y * f, n.z * f));
        }

        #[test]
        fn mul_assign_f32(n in normal3_f32(), f in range_f32()) {
            let mut n1 = n;
            n1 *= f;
            prop_assert_eq!(n1, normal3(n.x * f, n.y * f, n.z * f));
        }

        #[test]
        fn div_i32(
            n in normal3_i32(),
            f in (-100..100i32).prop_filter("non-zero", |x| *x != 0)
        ) {
            let s = 1 / f;
            prop_assert_eq!(n / f, normal3(n.x * s, n.y * s, n.z * s));
        }

        #[test]
        fn div_f32(n in normal3_f32(), f in non_zero_f32()) {
            let s = 1.0 / f;
            prop_assert_eq!(n / f, normal3(n.x * s, n.y * s, n.z * s));
        }

        #[test]
        fn div_assign_i32(n in normal3_i32(), f in non_zero_i32()) {
            let mut n1 = n;
            n1 /= f;

            let s = 1 / f;
            prop_assert_eq!(n1, normal3(n.x * s, n.y * s, n.z * s));
        }

        #[test]
        fn div_assign_f32(n in normal3_f32(), f in non_zero_f32()) {
            let mut n1 = n;
            n1 /= f;

            let s = 1.0 / f;
            prop_assert_eq!(n1, normal3(n.x * s, n.y * s, n.z * s));
        }

        #[test]
        fn neg_i32(n in normal3_i32()) {
            prop_assert_eq!(-n, normal3(-n.x, -n.y, -n.z));
            prop_assert_eq!(--n, n);
        }

        #[test]
        fn neg_f32(n in normal3_f32()) {
            prop_assert_eq!(-n, normal3(-n.x, -n.y, -n.z));
            prop_assert_eq!(--n, n);
        }

        #[test]
        fn index_i32(n in normal3_i32()) {
            prop_assert_eq!(n[Axis::X], n.x);
            prop_assert_eq!(n[Axis::Y], n.y);
        }

        #[test]
        fn index_f32(n in normal3_f32()) {
            prop_assert_eq!(n[Axis::X], n.x);
            prop_assert_eq!(n[Axis::Y], n.y);
        }

        #[test]
        fn index_mut_i32(n in normal3_i32()) {
            let mut n1 = normal3(-200, 200, -200);
            n1[Axis::X] = n.x;
            n1[Axis::Y] = n.y;
            n1[Axis::Z] = n.z;
            prop_assert_eq!(n1, n);
        }

        #[test]
        fn index_mut_f32(n in normal3_f32()) {
            let mut n1 = normal3(-200.0, 200.0, -200.0);
            n1[Axis::X] = n.x;
            n1[Axis::Y] = n.y;
            n1[Axis::Z] = n.z;
            prop_assert_eq!(n1, n);
        }
    }
}
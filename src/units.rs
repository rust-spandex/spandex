//! Set of all dimension units used by SpanDeX, along with conversion rules
//! to go from one to another easily.
//!
//! The main conversion rules used so far are that 1 in = 72.27 pt = 2.54 cm and 1 pt = 65,536 sp.
use core::ops::Neg;
use num_integer::Integer;
use num_traits::identities::{One, Zero};
use num_traits::{Num, Pow, Signed};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Rem, Sub, SubAssign};
use std::{f64, fmt};

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Measure of what is supposed to be positive infinity.
///
/// Any measure exceeding this value will be considered infinite.
pub const PLUS_INFINITY: Sp = Sp(10_000_000_000);

/// Measure of what is supposed to be negative infinity.
pub const MINUS_INFINITY: Sp = Sp(-10_000_000_000);

/// Scaled point, equal to 1/65,536 of a point.
///
/// Defining this unit is useful because the wavelength of visible light is around 100 sp. This
/// makes rounding errors invisible to the eye, which allows to perform uniquely integer
/// arithmetics by treating all dimensions as integer multiples of this tiny unit. This ensures
/// consistent computations, and thus output, across a wide variety of computers.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sp(pub i64);

/// Millimeters.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mm(pub f64);

/// Points.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Pt(pub f64);

impl Zero for Sp {
    fn zero() -> Self {
        Sp(0)
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl One for Sp {
    fn one() -> Self {
        Sp(1)
    }

    fn is_one(&self) -> bool {
        self.0 == 1
    }
}

impl Num for Sp {
    type FromStrRadixErr = std::num::ParseIntError;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        match i64::from_str_radix(str, radix) {
            Ok(parsed) => Ok(Sp(parsed)),
            Err(e) => Err(e),
        }
    }
}

impl Integer for Sp {
    fn div_floor(&self, other: &Self) -> Self {
        Sp(self.0.div_floor(&other.0))
    }

    fn mod_floor(&self, other: &Self) -> Self {
        Sp(self.0.mod_floor(&other.0))
    }

    fn gcd(&self, other: &Self) -> Self {
        Sp(self.0.gcd(&other.0))
    }

    fn lcm(&self, other: &Self) -> Self {
        Sp(self.0.lcm(&other.0))
    }

    fn divides(&self, other: &Self) -> bool {
        self.0.divides(&other.0)
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        self.0.is_multiple_of(&other.0)
    }

    fn is_even(&self) -> bool {
        self.0.is_even()
    }

    fn is_odd(&self) -> bool {
        self.0.is_odd()
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        let (div, rem) = self.0.div_rem(&other.0);

        (Sp(div), Sp(rem))
    }

    fn div_mod_floor(&self, other: &Self) -> (Self, Self) {
        let (div, rem) = self.0.div_mod_floor(&other.0);

        (Sp(div), Sp(rem))
    }
}

impl Neg for Sp {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Sp(0) - self
    }
}

impl Signed for Sp {
    fn abs(&self) -> Self {
        Sp(self.0.abs())
    }

    fn abs_sub(&self, other: &Self) -> Self {
        Sp(self.0.abs_sub(&other.0))
    }

    fn signum(&self) -> Self {
        Sp(self.0.signum())
    }

    fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    fn is_negative(&self) -> bool {
        self.0.is_negative()
    }
}

impl Pow<u32> for Sp {
    type Output = Sp;

    fn pow(self, rhs: u32) -> Self::Output {
        Sp(self.0.pow(rhs))
    }
}

impl fmt::Debug for Sp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} sp", self.0)
    }
}

impl fmt::Debug for Mm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} mm", self.0)
    }
}

impl fmt::Debug for Pt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} pt", self.0)
    }
}

macro_rules! impl_operators {
    ($the_type: ty, $constructor: expr) => {
        impl Add for $the_type {
            type Output = $the_type;

            fn add(self, other: $the_type) -> $the_type {
                $constructor(self.0 + other.0)
            }
        }

        impl AddAssign for $the_type {
            fn add_assign(&mut self, other: $the_type) {
                self.0 += other.0;
            }
        }

        impl Sub for $the_type {
            type Output = $the_type;

            fn sub(self, other: $the_type) -> $the_type {
                $constructor(self.0 - other.0)
            }
        }

        impl SubAssign for $the_type {
            fn sub_assign(&mut self, other: $the_type) {
                self.0 -= other.0;
            }
        }

        impl Div for $the_type {
            type Output = $the_type;

            fn div(self, other: $the_type) -> $the_type {
                $constructor(self.0 / other.0)
            }
        }

        impl DivAssign for $the_type {
            fn div_assign(&mut self, other: $the_type) {
                self.0 /= other.0;
            }
        }

        impl Mul for $the_type {
            type Output = $the_type;

            fn mul(self, other: $the_type) -> $the_type {
                $constructor(self.0 * other.0)
            }
        }

        impl Rem for $the_type {
            type Output = $the_type;

            fn rem(self, other: $the_type) -> $the_type {
                $constructor(self.0 % other.0)
            }
        }
    };
}

impl_operators!(Sp, Sp);
impl_operators!(Mm, Mm);
impl_operators!(Pt, Pt);

impl Mul<i64> for Sp {
    type Output = Sp;

    fn mul(self, rhs: i64) -> Sp {
        Sp(self.0 * rhs)
    }
}

impl Mul<Sp> for i64 {
    type Output = Sp;

    fn mul(self, rhs: Sp) -> Sp {
        Sp(self * rhs.0)
    }
}

impl PartialOrd for Sp {
    fn partial_cmp(&self, other: &Sp) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Sp {
    fn cmp(&self, other: &Sp) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl From<Mm> for Sp {
    fn from(mm: Mm) -> Sp {
        // 10 mm = 1 cm
        Sp(((72.27 / 25.4) * 65_536.0 * mm.0).round() as i64)
    }
}

impl Into<Mm> for Sp {
    fn into(self) -> Mm {
        Mm((25.4 / (72.27 * 65_536.0)) * (self.0 as f64))
    }
}

impl From<Pt> for Sp {
    fn from(pt: Pt) -> Sp {
        Sp((65_536.0 * pt.0).round() as i64)
    }
}

impl Into<Pt> for Sp {
    fn into(self) -> Pt {
        Pt((self.0 as f64) / 65_536.0)
    }
}

impl From<Pt> for Mm {
    fn from(pt: Pt) -> Mm {
        Mm((25.4 / 72.27) * pt.0)
    }
}

impl Into<Pt> for Mm {
    fn into(self) -> Pt {
        Pt((72.27 / 25.4) * self.0)
    }
}

/// Compares two float numbers to check if they're close enough to be
/// considered equal.
///
/// Inspired by [this post](https://users.rust-lang.org/t/assert-eq-for-float-numbers/7034/3).
///
/// # Examples
///
/// ```
/// # use spandex::units::nearly_equal;
/// assert_eq!(nearly_equal(3.0, 2.99999), true);
/// assert_eq!(nearly_equal(4.0, 3.999), false);
/// ```
pub fn nearly_equal(a: f64, b: f64) -> bool {
    let abs_a = a.abs();
    let abs_b = b.abs();
    let diff = (a - b).abs();

    if a == b {
        // Handle infinities.
        true
    } else if a == 0.0 || b == 0.0 || diff < f64::MIN_POSITIVE {
        // One of a or b is zero (or both are extremely close to it,) use absolute error.
        diff < (f64::EPSILON * f64::MIN_POSITIVE)
    } else {
        // Use relative error.
        (diff / f64::min(abs_a + abs_b, f64::MAX)) < 10e-5
    }
}

/// Unit tests for SpanDeX.
#[cfg(test)]
mod tests {
    use crate::units::{nearly_equal, Mm, Pt, Sp};

    #[test]
    fn convert_mm_to_sp() {
        let expected_result = Sp(3_263_190);
        let size_in_mm = Mm(17.5);
        let cast_value = Sp::from(size_in_mm);
        assert_eq!(cast_value, expected_result);
    }

    #[test]
    fn convert_pt_to_sp() {
        let expected_result = Sp(3_723_756);
        let size_in_pt = Pt(56.82);
        let cast_from_sp = Sp::from(size_in_pt);
        assert_eq!(cast_from_sp, expected_result);
    }

    #[test]
    fn convert_sp_to_mm() {
        let expected_result = Mm(17.5);
        let size_in_sp = Sp(3_263_189);
        let cast_from_sp: Mm = size_in_sp.into();
        assert!(nearly_equal(cast_from_sp.0, expected_result.0));
    }

    #[test]
    fn convert_sp_to_pt() {
        let expected_result = Pt(20.25);
        let size_in_sp = Sp(1_327_104);
        let cast_from_sp: Pt = size_in_sp.into();
        assert!(nearly_equal(cast_from_sp.0, expected_result.0));
    }

    #[test]
    fn convert_pt_to_mm() {
        let expected_result = Mm(4.481);
        let size_in_pt = Pt(12.75);
        let cast_from_pt = Mm::from(size_in_pt);
        assert!(nearly_equal(cast_from_pt.0, expected_result.0));
    }

    #[test]
    fn convert_mm_to_pt() {
        let expected_result = Pt(12.75);
        let size_in_mm = Mm(4.481);
        let cast_from_mm: Pt = size_in_mm.into();
        assert!(nearly_equal(cast_from_mm.0, expected_result.0));
    }
}

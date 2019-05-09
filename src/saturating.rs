//! This module contains a useful trait to deal with saturating operations.

use num_rational::Ratio;
use num_integer::Integer;

use crate::units::Sp;

/// A trait that provides saturating operators.
pub trait Saturating {
    /// A saturating addition.
    fn sadd(&self, rhs: &Self) -> Self;

    /// A saturating product.
    fn smul(&self, rhs: &Self) -> Self;

    /// A saturating exponentiation.
    fn spow(&self, rhs: u32) -> Self;
}

macro_rules! impl_saturating {
    ($ty: ty) => {
        impl Saturating for $ty {
            fn sadd(&self, rhs: &Self) -> Self {
                self.saturating_add(*rhs)
            }

            fn smul(&self, rhs: &Self) -> Self {
                self.saturating_mul(*rhs)
            }

            fn spow(&self, rhs: u32) -> Self {
                self.saturating_pow(rhs)
            }
        }
    }
}

impl_saturating!(i8);
impl_saturating!(i16);
impl_saturating!(i32);
impl_saturating!(i64);
impl_saturating!(isize);
impl_saturating!(u8);
impl_saturating!(u16);
impl_saturating!(u32);
impl_saturating!(u64);
impl_saturating!(usize);

impl Saturating for Sp {
    fn sadd(&self, rhs: &Self) -> Self {
        Sp(self.0.sadd(&rhs.0))
    }
    fn smul(&self, rhs: &Self) -> Self {
        Sp(self.0.smul(&rhs.0))
    }
    fn spow(&self, rhs: u32) -> Self {
        Sp(self.0.spow(rhs))
    }
}

impl<T: Clone + Integer + Saturating> Saturating for Ratio<T> {
    fn sadd(&self, rhs: &Self) -> Self {
        let mut numer = self.numer().clone();
        let mut denom = self.denom().clone();
        numer = numer.smul(rhs.denom());
        numer = numer.sadd(&denom.smul(rhs.numer()));
        denom = denom.smul(rhs.denom());

        Ratio::new(numer, denom)
    }
    fn smul(&self, rhs: &Self) -> Self {
        Ratio::new(
            self.numer().clone().smul(rhs.numer()),
            self.denom().clone().smul(rhs.denom()),
        )
    }
    fn spow(&self, rhs: u32) -> Self {
        Ratio::new(
            self.numer().clone().spow(rhs),
            self.denom().clone().spow(rhs),
        )
    }
}

#[cfg(test)]
mod tests {

    use num_rational::Ratio;
    use crate::saturating::Saturating;
    use crate::units::Sp;

    #[test]
    fn test_add() {
        let a = Ratio::new(1, 2);
        let b = Ratio::new(1, 4);
        assert_eq!(a.sadd(&b), Ratio::new(3, 4));
    }

    #[test]
    fn test_add_sp() {
        let a = Ratio::new(Sp(1), Sp(2));
        let b = Ratio::new(Sp(1), Sp(4));
        assert_eq!(a.sadd(&b), Ratio::new(Sp(3), Sp(4)));
    }

    #[test]
    fn test_pow() {
        let r = Ratio::<i64>::new(i64::max_value(), 1);
        let p = r.spow(3);
        assert_eq!(p, r);

        let r = Ratio::<i64>::new(1, 3);
        assert_eq!(r.spow(2), Ratio::<i64>::new(1, 9));
        assert_eq!(r.spow(3), Ratio::<i64>::new(1, 27));
    }

}

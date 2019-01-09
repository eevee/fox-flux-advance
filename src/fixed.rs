/// Fixed-point fractional type.
///
/// For various reasons, existing crates won't work.

use euclid::num::{Ceil, Floor, Round, One, Zero};

use core::cmp;
use core::ops;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub struct Fixed(FixedStore);

type FixedStore = i32;
type FixedWhole = i16;
type FixedFraction = u16;

const TILE_SIZE_BITS: usize = 3;

impl Fixed {
    pub const FRACTIONAL_BITS: usize = 16;
    pub const PRECISION: FixedFraction = 1;
    pub const MAX_FRACTION: FixedFraction = ((1usize << Self::FRACTIONAL_BITS) - 1) as FixedFraction;

    pub fn promote(n: FixedWhole) -> Self {
        Self((n as FixedStore) << Self::FRACTIONAL_BITS)
    }

    pub fn max_fraction() -> Self {
        Self(Self::MAX_FRACTION as FixedStore)
    }

    pub fn to_int_floor(self) -> FixedWhole {
        (self.0 >> Self::FRACTIONAL_BITS) as FixedWhole
    }

    pub fn to_int_round(self) -> FixedWhole {
        (self + Self::max_fraction()).to_int_floor()
    }

    // TODO maybe this is more appropriate on a typed Length
    pub fn to_tile_coord(self) -> usize {
        // TODO what if i'm negative
        (self.0 >> (Self::FRACTIONAL_BITS + TILE_SIZE_BITS)) as usize
    }

    // TODO maybe this is more appropriate on a typed Length
    pub fn to_sprite_offset_x(self) -> u16 {
        // TODO what if i'm too big or small
        (self.to_int_round() + 512) as u16 & 0x01ffu16
    }
    pub fn to_sprite_offset_y(self) -> u16 {
        // TODO what if i'm too big or small
        (self.to_int_round() + 256) as u16 & 0x00ffu16
    }
}


// Standard comparison traits

impl cmp::PartialEq<FixedWhole> for Fixed {
    fn eq(&self, other: &FixedWhole) -> bool {
        *self == Self::promote(*other)
    }
}

impl cmp::PartialOrd<FixedWhole> for Fixed {
    fn partial_cmp(&self, other: &FixedWhole) -> Option<cmp::Ordering> {
        Some(self.0.cmp(&Self::promote(*other).0))
    }
}


// Standard math traits (other fixeds)

impl ops::Neg for Fixed {
    type Output = Self;

    fn neg(self) -> Self {
        Fixed(-self.0)
    }
}

impl ops::Add<Fixed> for Fixed {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Fixed(self.0 + other.0)
    }
}

impl ops::AddAssign<Fixed> for Fixed {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl ops::Sub<Fixed> for Fixed {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Fixed(self.0 - other.0)
    }
}

impl ops::SubAssign<Fixed> for Fixed {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl ops::Div<Fixed> for Fixed {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        // FIXME what about fractional bits...?
        Self::promote((self.0 / other.0) as i16)
    }
}

impl ops::DivAssign<Fixed> for Fixed {
    fn div_assign(&mut self, other: Self) {
        *self = *self / other;
    }
}

impl ops::Rem<Fixed> for Fixed {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        Fixed(self.0 % other.0)
    }
}

impl ops::RemAssign<Fixed> for Fixed {
    fn rem_assign(&mut self, other: Self) {
        self.0 %= other.0
    }
}


// Standard math traits (ints)

impl ops::Add<FixedWhole> for Fixed {
    type Output = Fixed;

    fn add(self, other: FixedWhole) -> Self {
        self + Self::promote(other)
    }
}

impl ops::Add<Fixed> for FixedWhole {
    type Output = Fixed;

    fn add(self, other: Fixed) -> Fixed {
        Fixed::promote(self) + other
    }
}

impl ops::AddAssign<FixedWhole> for Fixed {
    fn add_assign(&mut self, other: FixedWhole) {
        *self += Self::promote(other)
    }
}

impl ops::Sub<FixedWhole> for Fixed {
    type Output = Fixed;

    fn sub(self, other: FixedWhole) -> Self {
        self - Self::promote(other)
    }
}

impl ops::Sub<Fixed> for FixedWhole {
    type Output = Fixed;

    fn sub(self, other: Fixed) -> Fixed {
        Fixed::promote(self) - other
    }
}

impl ops::SubAssign<FixedWhole> for Fixed {
    fn sub_assign(&mut self, other: FixedWhole) {
        *self -= Self::promote(other)
    }
}

impl ops::Div<FixedWhole> for Fixed {
    type Output = Self;

    fn div(self, other: FixedWhole) -> Self {
        Self(self.0 / other as FixedStore)
    }
}

impl ops::DivAssign<FixedWhole> for Fixed {
    fn div_assign(&mut self, other: FixedWhole) {
        self.0 /= other as FixedStore;
    }
}

impl ops::Rem<FixedWhole> for Fixed {
    type Output = Self;

    fn rem(self, other: FixedWhole) -> Self {
        self % Self::promote(other)
    }
}

impl ops::RemAssign<FixedWhole> for Fixed {
    fn rem_assign(&mut self, other: FixedWhole) {
        *self %= Self::promote(other)
    }
}


// Standard conversion traits

impl From<FixedWhole> for Fixed {
    fn from(whole: FixedWhole) -> Self {
        Self::promote(whole)
    }
}


// Euclid traits

impl Zero for Fixed {
    fn zero() -> Self {
        Self::promote(0)
    }
}

impl One for Fixed {
    fn one() -> Self {
        Self::promote(1)
    }
}

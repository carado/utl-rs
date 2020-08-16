use {std::{ops::{Shr, Shl}, mem::size_of}, num_traits::{PrimInt, Saturating}};

pub trait BitsExt:
	PrimInt + Shl<u8, Output = Self> + Shr<u8, Output = Self> + Saturating
{
	const BIT_COUNT: u8 = (8 * size_of::<Self>()) as u8;

	fn log2ceil(self) -> u8 {
		//Self::BIT_COUNT - self.saturating_sub(Self::one()).leading_zeros() as u8
		Self::BIT_COUNT - self.leading_zeros() as u8
	}

	fn ceil_div(self, y: Self) -> Self { (self + y - Self::one()) / y }

	fn ceil_shr(self, y: u8) -> Self {
		(self + (Self::one() << y) - Self::one()) >> y
	}

	fn log2(self) -> Option<u8> {
		let l2 = self.log2ceil();
		if Self::one() << l2 == self {
			Some(l2)
		} else {
			None
		}
	}
}

impl<T> BitsExt for T where
	T: PrimInt + Shl<u8, Output = Self> + Shr<u8, Output = Self> + Saturating
{}

#[test]
fn test() {
	assert!(0u8.log2ceil() == 0);
	assert!(1u8.log2ceil() == 0);

	assert!(7u8.log2ceil() == 3);
	assert!(8u8.log2ceil() == 3);
	assert!(9u8.log2ceil() == 4);

	assert!(254u8.log2ceil() == 8);
	assert!(255u8.log2ceil() == 8);
}


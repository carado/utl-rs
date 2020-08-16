use {std::{ops::{Shr, Shl}, mem::size_of}, num_traits::{PrimInt, Saturating}};

pub trait BitsExt:
	PrimInt + Shl<u32, Output = Self> + Shr<u32, Output = Self> + Saturating
{
	const BIT_COUNT: u32 = (8 * size_of::<Self>()) as u32;

	fn log2ceil(self) -> u32 {
		//Self::BIT_COUNT - self.saturating_sub(Self::one()).leading_zeros() as u32
		Self::BIT_COUNT - self.leading_zeros() as u32
	}

	fn ceil_div(self, y: Self) -> Self { (self + y - Self::one()) / y }

	fn ceil_shr(self, y: u32) -> Self {
		(self + (Self::one() << y) - Self::one()) >> y
	}

	fn log2(self) -> Option<u32> {
		let l2 = self.log2ceil();
		if Self::one() << l2 == self {
			Some(l2)
		} else {
			None
		}
	}

	fn pad_to_align(self, align: Self) -> Self {
		debug_assert_eq!(align.count_ones(), 1);
		let align_l2 = align.trailing_zeros();
		self.ceil_shr(align_l2) << align_l2
	}
}

impl<T> BitsExt for T where
	T: PrimInt + Shl<u32, Output = Self> + Shr<u32, Output = Self> + Saturating
{}

#[test]
fn test() {
	assert!(0u32.log2ceil() == 0);
	assert!(1u32.log2ceil() == 0);

	assert!(7u32.log2ceil() == 3);
	assert!(8u32.log2ceil() == 3);
	assert!(9u32.log2ceil() == 4);

	assert!(254u32.log2ceil() == 8);
	assert!(255u32.log2ceil() == 8);
}


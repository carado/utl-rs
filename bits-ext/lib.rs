use {std::{ops::{Shr, Shl}, mem::size_of}, num_traits::{PrimInt, Saturating}};

pub trait BitsExt:
	PrimInt + Shl<u32, Output = Self> + Shr<u32, Output = Self> + Saturating
{
	const BIT_COUNT: u32 = (8 * size_of::<Self>()) as u32;

	#[inline]
	fn log2ceil(self) -> Option<u32> {
		if self == Self::zero() {
			None
		} else {
			Some(Self::BIT_COUNT - (self - Self::one()).leading_zeros())
		}
	}

	#[inline]
	fn bit_len(self) -> u32 {
		Self::BIT_COUNT - self.leading_zeros()
	}

	#[inline]
	fn ceil_div(self, y: Self) -> Self { (self + y - Self::one()) / y }

	#[inline]
	fn ceil_shr(self, y: u32) -> Self {
		(self + (Self::one() << y) - Self::one()) >> y
	}

	#[inline]
	fn log2(self) -> Option<u32> {
		let l2 = self.log2ceil()?;
		if Self::one() << l2 == self {
			Some(l2)
		} else {
			None
		}
	}

	#[inline]
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
	{
		assert_eq!(0u32.bit_len(), 0);

		assert_eq!(1u32.bit_len(), 1);

		assert_eq!(2u32.bit_len(), 2);
		assert_eq!(3u32.bit_len(), 2);

		assert_eq!(4u32.bit_len(), 3);
		assert_eq!(5u32.bit_len(), 3);
		assert_eq!(6u32.bit_len(), 3);
		assert_eq!(7u32.bit_len(), 3);

		assert_eq!(8u32.bit_len(), 4);
		assert_eq!(9u32.bit_len(), 4);

		assert_eq!(254u32.bit_len(), 8);
		assert_eq!(255u32.bit_len(), 8);
		assert_eq!(256u32.bit_len(), 9);
		assert_eq!(257u32.bit_len(), 9);
	}

	{
		assert_eq!(0u32.log2ceil(), None);

		assert_eq!(1u32.log2ceil(), Some(0));

		assert_eq!(2u32.log2ceil(), Some(1));

		assert_eq!(3u32.log2ceil(), Some(2));
		assert_eq!(4u32.log2ceil(), Some(2));

		assert_eq!(5u32.log2ceil(), Some(3));
		assert_eq!(6u32.log2ceil(), Some(3));
		assert_eq!(7u32.log2ceil(), Some(3));
		assert_eq!(8u32.log2ceil(), Some(3));
		assert_eq!(9u32.log2ceil(), Some(4));

		assert_eq!(254u32.log2ceil(), Some(8));
		assert_eq!(255u32.log2ceil(), Some(8));
		assert_eq!(256u32.log2ceil(), Some(8));
		assert_eq!(257u32.log2ceil(), Some(9));
	}
}


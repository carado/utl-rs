use {
	std::{hash::Hash, fmt::{Debug, Display}, ops::{BitOr, BitOrAssign}},
	num_traits::PrimInt,
};

pub mod cast_prims {
	use num_traits::AsPrimitive as As;

	pub trait CastPrims =
		As<u8> + As<u16> + As<u32> + As<u64> + As<u128> + As<usize> +
		As<i8> + As<i16> + As<i32> + As<i64> + As<i128> + As<isize> +
		As<f32> + As<f64>
	where
		u8: As<Self>, u16: As<Self>, u32: As<Self>, u64: As<Self>, u128: As<Self>,
		usize: As<Self>,
		i8: As<Self>, i16: As<Self>, i32: As<Self>, i64: As<Self>, i128: As<Self>,
		isize: As<Self>,
		f32: As<Self>, f64: As<Self>, bool: As<Self>,
	;
}

pub unsafe trait NonZero: 'static
	+ Sized + Copy + Debug + Display
	+ Hash + Eq + Ord
	+ BitOr + BitOrAssign
{
	type Raw: num_traits::PrimInt + From<Self> + cast_prims::CastPrims;
	fn non_zero_from(raw: Self::Raw) -> Option<Self>;
	unsafe fn non_zero_from_unchecked(raw: Self::Raw) -> Self;
}

pub trait ToNonZero: num_traits::PrimInt + cast_prims::CastPrims {
	type NonZero: NonZero<Raw = Self>;
	fn to_non_zero(self) -> Option<Self::NonZero>;
	unsafe fn to_zero_from_unchecked(self) -> Self::NonZero;
}

//impl<T: NonZero> HasNonZero for T::

macro_rules! impl_non_zero{
	($($nz:ident $raw:ident,)*) => {
		pub use std::num::{
			$(
				$nz as $raw,
			)*
		};

		mod impls {
			$(
				unsafe impl super::NonZero for std::num::$nz {
					type Raw = $raw;

					fn non_zero_from(raw: Self::Raw) -> Option<Self> {
						std::num::$nz::new(raw)
					}

					unsafe fn non_zero_from_unchecked(raw: Self::Raw) -> Self {
						std::num::$nz::new_unchecked(raw)
					}
				}
			)*
		}
	};
}

impl_non_zero!{
	NonZeroU8 u8, NonZeroU16 u16, NonZeroU32 u32, NonZeroU64 u64,
	NonZeroU128 u128, NonZeroUsize usize,
	NonZeroI8 i8, NonZeroI16 i16, NonZeroI32 i32, NonZeroI64 i64,
	NonZeroI128 i128, NonZeroIsize isize,
}


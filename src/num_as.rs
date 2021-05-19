pub trait NumAs<T> {
	fn num_as(self) -> T;
	fn try_num_as(self) -> Option<T>;
}

impl<T, U> NumAs<U> for T where
	T: num_traits::AsPrimitive<U>,
	U: 'static + Copy + std::convert::TryFrom<T>,
{
	#[cfg(debug_assertions)]
	#[inline]
	fn num_as(self) -> U {
		self.try_num_as().expect("failed num_as")
	}

	#[cfg(not(debug_assertions))]
	#[inline]
	fn num_as(self) -> U {
		self.as_()
	}

	fn try_num_as(self) -> Option<U> { U::try_from(self).ok() }
}


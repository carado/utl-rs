#[cfg(debug_assertions)]
macro_rules! unsafe_unreachable{ () => { unreachable!() }; }

#[cfg(not(debug_assertions))]
macro_rules! unsafe_unreachable{ () => { std::hint::unreachable_unchecked() }; }

pub trait UnsafeUnwrap {
	type Value;
	unsafe fn unsafe_unwrap(self) -> Self::Value;
	unsafe fn unsafe_unwrap_ref(&self) -> &Self::Value;
	unsafe fn unsafe_unwrap_mut(&mut self) -> &mut Self::Value;
}

impl<T> UnsafeUnwrap for Option<T> {
	type Value = T;

	#[inline]
	unsafe fn unsafe_unwrap(self) -> T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable!(),
		}
	}

	#[inline]
	unsafe fn unsafe_unwrap_ref(&self) -> &T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable!(),
		}
	}

	#[inline]
	unsafe fn unsafe_unwrap_mut(&mut self) -> &mut T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable!(),
		}
	}
}

impl<T, E> UnsafeUnwrap for Result<T, E> {
	type Value = T;

	#[inline]
	unsafe fn unsafe_unwrap(self) -> T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable!(),
		}
	}

	#[inline]
	unsafe fn unsafe_unwrap_ref(&self) -> &T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable!(),
		}
	}

	#[inline]
	unsafe fn unsafe_unwrap_mut(&mut self) -> &mut T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable!(),
		}
	}
}


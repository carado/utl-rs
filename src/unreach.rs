#[cfg(debug_assertions)]
macro_rules! unsafe_unreachable{ () => { unreachable!() }; }

#[cfg(not(debug_assertions))]
macro_rules! unsafe_unreachable{ () => { std::hint::unreachable_unchecked() }; }

pub trait UnsafeUnwrap {
	type Value;
	type Alt;
	unsafe fn unsafe_unwrap(self) -> Self::Value;
	unsafe fn unsafe_unwrap_ref(&self) -> &Self::Value;
	unsafe fn unsafe_unwrap_mut(&mut self) -> &mut Self::Value;
	unsafe fn unsafe_unwrap_alt(self) -> Self::Alt;
}

impl<T> UnsafeUnwrap for Option<T> {
	type Value = T;
	type Alt = ();

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

	#[inline]
	unsafe fn unsafe_unwrap_alt(self) {
		match self {
			Some(_) => unsafe_unreachable!(),
			None => {},
		}
	}
}

impl<T, E> UnsafeUnwrap for Result<T, E> {
	type Value = T;
	type Alt = E;

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

	#[inline]
	unsafe fn unsafe_unwrap_alt(self) -> E {
		match self {
			Ok(_) => unsafe_unreachable!(),
			Err(e) => e,
		}
	}
}


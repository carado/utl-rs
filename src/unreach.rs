#[cfg(debug_assertions)]
#[deprecated(note = "use unsafe_unreachable (function rather than macro)")]
#[macro_export]
macro_rules! unsafe_unreachable{ () => { unreachable!() }; }

#[cfg(not(debug_assertions))]
#[deprecated(note = "use unsafe_unreachable (function rather than macro)")]
#[macro_export]
macro_rules! unsafe_unreachable{ () => { std::hint::unreachable_unchecked() }; }

#[cfg(debug_assertions)]
#[inline(always)]
pub unsafe fn unsafe_unreachable() -> ! { unreachable!() }

#[cfg(not(debug_assertions))]
#[inline(always)]
pub unsafe fn unsafe_unreachable() -> ! { std::hint::unreachable_unchecked() }

pub use self::unsafe_unreachable as unreachable_unchecked;

pub trait UnsafeUnwrap {
	type Value;
	type Alt;
	//unsafe fn unwrap_unchecked(self) -> Self::Value;
	unsafe fn unwrap_unchecked_ref(&self) -> &Self::Value;
	unsafe fn unwrap_unchecked_mut(&mut self) -> &mut Self::Value;
	//unsafe fn unwrap_unchecked_alt(self) -> Self::Alt;
}

impl<T> UnsafeUnwrap for Option<T> {
	type Value = T;
	type Alt = ();

	/*
	#[inline]
	unsafe fn unwrap_unchecked(self) -> T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable(),
		}
	}
	*/

	#[inline]
	unsafe fn unwrap_unchecked_ref(&self) -> &T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable(),
		}
	}

	#[inline]
	unsafe fn unwrap_unchecked_mut(&mut self) -> &mut T {
		match self {
			Some(val) => val,
			None => unsafe_unreachable(),
		}
	}

	/*
	#[inline]
	unsafe fn unwrap_unchecked_alt(self) {
		match self {
			Some(_) => unsafe_unreachable(),
			None => {},
		}
	}
	*/
}

impl<T, E> UnsafeUnwrap for Result<T, E> {
	type Value = T;
	type Alt = E;

	/*
	#[inline]
	unsafe fn unwrap_unchecked(self) -> T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable(),
		}
	}
	*/

	#[inline]
	unsafe fn unwrap_unchecked_ref(&self) -> &T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable(),
		}
	}

	#[inline]
	unsafe fn unwrap_unchecked_mut(&mut self) -> &mut T {
		match self {
			Ok(val) => val,
			Err(_) => unsafe_unreachable(),
		}
	}

	/*
	#[inline]
	unsafe fn unwrap_unchecked_alt(self) -> E {
		match self {
			Ok(_) => unsafe_unreachable(),
			Err(e) => e,
		}
	}
	*/
}


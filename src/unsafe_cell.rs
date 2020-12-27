macro_rules! base_impl {
	($cell:ident $ref:ident $mut:ident) => {
		impl<T> $cell<T> {
			pub fn new(value: T) -> Self { Self(value.into()) }
			pub fn into_inner(self) -> T { self.0.into_inner() }
		}

		impl<T> From<T> for $cell<T> {
			fn from(value: T) -> Self { Self(value.into()) }
		}

		impl<T: ?Sized> std::ops::Deref for $ref<'_, T> {
			type Target = T;
			fn deref(&self) -> &T { &self.0 }
		}

		impl<T: ?Sized> std::ops::Deref for $mut<'_, T> {
			type Target = T;
			fn deref(&self) -> &T { &self.0 }
		}

		impl<T: ?Sized> std::ops::DerefMut for $mut<'_, T> {
			fn deref_mut(&mut self) -> &mut T { &mut self.0 }
		}
	};
}

pub mod debug {
	#[derive(Default, Debug)]
	pub struct UnsafeCell<T: ?Sized>(parking_lot::RwLock<T>);

	pub struct UnsafeCellRef<'a, T: ?Sized>(parking_lot::RwLockReadGuard<'a, T>);

	pub struct UnsafeCellMut<'a, T: ?Sized>(parking_lot::RwLockWriteGuard<'a, T>);

	base_impl!(UnsafeCell UnsafeCellRef UnsafeCellMut);

	impl<T: ?Sized> UnsafeCell<T> {
		pub unsafe fn get(&self) -> UnsafeCellRef<T> {
			UnsafeCellRef(
				self.0.try_read().expect("debug UnsafeCell caught data race")
			)
		}

		pub unsafe fn get_mut(&self) -> UnsafeCellMut<T> {
			UnsafeCellMut(
				self.0.try_write().expect("debug UnsafeCell caught data race")
			)
		}
	}
}

pub mod release {
	#[derive(Default, Debug)]
	pub struct UnsafeCell<T: ?Sized>(std::cell::UnsafeCell<T>);

	unsafe impl<T: Sync> Sync for UnsafeCell<T> {}
	unsafe impl<T: Send> Send for UnsafeCell<T> {}

	pub struct UnsafeCellRef<'a, T: ?Sized>(&'a T);

	pub struct UnsafeCellMut<'a, T: ?Sized>(&'a mut T);

	base_impl!(UnsafeCell UnsafeCellRef UnsafeCellMut);

	impl<T: ?Sized> UnsafeCell<T> {
		pub unsafe fn get(&self) -> UnsafeCellRef<T> {
			UnsafeCellRef(&*self.0.get())
		}

		pub unsafe fn get_mut(&self) -> UnsafeCellMut<T> {
			UnsafeCellMut(&mut *self.0.get())
		}
	}
}

#[cfg(debug_assertions)]
pub use self::debug::*;

#[cfg(not(debug_assertions))]
pub use self::release::*;


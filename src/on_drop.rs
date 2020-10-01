use std::{ops::{Deref, DerefMut}, mem::ManuallyDrop};

pub struct OnDrop<T, F: FnOnce(T)>(ManuallyDrop<T>, ManuallyDrop<F>);

impl<T, F: FnOnce(T)> OnDrop<T, F> {
	pub fn new(value: T, fn_: F) -> Self {
		OnDrop(ManuallyDrop::new(value), ManuallyDrop::new(fn_))
	}
}

impl<T, F: FnOnce(T)> Deref for OnDrop<T, F> {
	type Target = T;
	fn deref(&self) -> &T { &self.0 }
}

impl<T, F: FnOnce(T)> DerefMut for OnDrop<T, F> {
	fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T, F: FnOnce(T)> Drop for OnDrop<T, F> {
	fn drop(&mut self) {
		unsafe { ManuallyDrop::take(&mut self.1)(ManuallyDrop::take(&mut self.0)); }
	}
}


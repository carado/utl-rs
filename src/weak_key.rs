use {
	std::{hash::{Hash, Hasher}, ops::CoerceUnsized},
	derive_more::{Deref, DerefMut},
};

#[derive(Clone, Deref, DerefMut)]
pub struct WeakKey<T: ?Sized>(pub T);

impl<T> WeakKey<T> {
	pub fn new(v: T) -> Self { Self(v) }

	pub fn into_inner(self) -> T { self.0 }
}

impl<T> PartialEq for WeakKey<std::sync::Weak<T>> {
	fn eq(&self, rhs: &Self) -> bool { self.0.ptr_eq(&rhs.0) }
	fn ne(&self, rhs: &Self) -> bool { self.0.as_ptr() != rhs.0.as_ptr() }
}

impl<T> Eq for WeakKey<std::sync::Weak<T>> {}

impl<T> Hash for WeakKey<std::sync::Weak<T>> {
	fn hash<H: Hasher>(&self, h: &mut H) { self.0.as_ptr().hash(h); }
}

impl<T> PartialEq for WeakKey<std::rc::Weak<T>> {
	fn eq(&self, rhs: &Self) -> bool { self.0.ptr_eq(&rhs.0) }
	fn ne(&self, rhs: &Self) -> bool { self.0.as_ptr() != rhs.0.as_ptr() }
}

impl<T> Eq for WeakKey<std::rc::Weak<T>> {}

impl<T> Hash for WeakKey<std::rc::Weak<T>> {
	fn hash<H: Hasher>(&self, h: &mut H) { self.0.as_ptr().hash(h); }
}

impl<T, U> CoerceUnsized<WeakKey<U>> for WeakKey<T> where
	T: ?Sized + CoerceUnsized<U>,
	U: ?Sized,
{}


use std::{cmp::Ordering, hash::{Hash, Hasher}, ops::{Deref, DerefMut, CoerceUnsized}};

#[derive(Debug, Copy, Clone)]
pub struct ByPtr<T: ?Sized>(pub T);

impl<T> ByPtr<T> {
	pub fn new(v: T) -> Self { Self(v) }

	pub fn into_inner(self) -> T { self.0 }
}

impl<T: ?Sized + Deref> ByPtr<T> {
	pub fn as_ptr(v: &Self) -> *const T::Target { (&***v) as _ }
}

impl<T: ?Sized + Deref> PartialEq for ByPtr<T> {
	fn eq(&self, o: &Self) -> bool { Self::as_ptr(self) == Self::as_ptr(o) }
	fn ne(&self, o: &Self) -> bool { Self::as_ptr(self) != Self::as_ptr(o) }
}

impl<T: ?Sized + Deref> Eq for ByPtr<T> {}

impl<T: ?Sized + Deref> Hash for ByPtr<T> {
	fn hash<H: Hasher>(&self, hasher: &mut H) { Self::as_ptr(self).hash(hasher) }
}

impl<T: ?Sized + Deref> PartialOrd for ByPtr<T> {
	fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
		Some(Self::as_ptr(&self).cmp(&Self::as_ptr(&o)))
	}

	fn lt(&self, o: &Self) -> bool { Self::as_ptr(self).lt(&Self::as_ptr(o)) }
	fn le(&self, o: &Self) -> bool { Self::as_ptr(self).le(&Self::as_ptr(o)) }
	fn gt(&self, o: &Self) -> bool { Self::as_ptr(self).gt(&Self::as_ptr(o)) }
	fn ge(&self, o: &Self) -> bool { Self::as_ptr(self).ge(&Self::as_ptr(o)) }
}

impl<T: ?Sized + Deref> Ord for ByPtr<T> {
	fn cmp(&self, o: &Self) -> Ordering {
		Self::as_ptr(self).cmp(&Self::as_ptr(o))
	}
}

impl<T> From<T> for ByPtr<T> {
	fn from(v: T) -> Self { Self(v) }
}

impl<T: ?Sized> Deref for ByPtr<T> {
	type Target = T;
	fn deref(&self) -> &T { &self.0 }
}

impl<T: ?Sized> DerefMut for ByPtr<T> {
	fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}

impl<T, U> CoerceUnsized<ByPtr<U>> for ByPtr<T> where
	T: ?Sized + CoerceUnsized<U>,
	U: ?Sized,
{}


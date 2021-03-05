pub use std::ops::{Deref, DerefMut};

pub trait DoubleDeref<T> = Deref where
	<Self as Deref>::Target: Deref<Target = T>;

pub trait DoubleDerefMut<T> = DerefMut where
	<Self as Deref>::Target: DerefMut<Target = T>;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Default, PartialOrd, Ord, Debug)]
pub struct DerefInner<T: ?Sized>(pub T);

impl<T> From<T> for DerefInner<T> {
	fn from(v: T) -> Self { Self(v) }
}

impl<T> Deref for DerefInner<T> where
	T: Deref,
	T::Target: Deref,
{
	type Target = <<T as Deref>::Target as Deref>::Target;

	#[inline]
	fn deref(&self) -> &Self::Target { &**self.0 }
}

impl<T> DerefMut for DerefInner<T> where
	T: DerefMut,
	T::Target: DerefMut,
{
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut **self.0 }
}


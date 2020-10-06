pub use std::ops::{Deref, DerefMut};

pub trait DoubleDeref<T> = Deref where
	<Self as Deref>::Target: Deref<Target = T>;

pub trait DoubleDerefMut<T> = DerefMut where
	<Self as Deref>::Target: DerefMut<Target = T>;


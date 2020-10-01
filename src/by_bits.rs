use std::{cmp::Ordering, hash::{Hash, Hasher}};

#[derive(Clone, Copy)]
pub struct ByBits<T>(pub T);

impl<T> From<T> for ByBits<T> {
	fn from(v: T) -> Self { Self(v) }
}

impl<T> std::ops::Deref for ByBits<T> {
	type Target = T;
	fn deref(&self) -> &T { &self.0 }
}

macro_rules! impl_by_bits {
	($ty:ty) => {
		impl PartialEq for ByBits<$ty> {
			fn eq(&self, rhs: &Self) -> bool { self.0.to_bits() == rhs.0.to_bits() }
			fn ne(&self, rhs: &Self) -> bool { self.0.to_bits() != rhs.0.to_bits() }
		}

		impl Eq for ByBits<$ty> {}

		impl Hash for ByBits<$ty> {
			fn hash<H: Hasher>(&self, h: &mut H) { self.0.to_bits().hash(h); }
		}

		impl PartialOrd for ByBits<$ty> {
			fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
				self.0.to_bits().partial_cmp(&rhs.0.to_bits())
			}
		}

		impl Ord for ByBits<$ty> {
			fn cmp(&self, rhs: &Self) -> Ordering {
				self.0.to_bits().cmp(&rhs.0.to_bits())
			}
		}
	}
}

impl_by_bits!{f32}
impl_by_bits!{f64}
impl_by_bits!{::half::f16}



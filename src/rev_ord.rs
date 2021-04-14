use std::{ops::{Deref, DerefMut}, cmp::Ordering};

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
#[deprecated(note = "use std::cmp:::Reverse")]
pub struct RevOrd<T: ?Sized>(pub T);

impl<T> RevOrd<T> {
	pub fn new(value: T) -> Self { RevOrd(value) }

	pub fn into_inner(self) -> T { self.0 }
}

impl<T: ?Sized + PartialOrd> PartialOrd for RevOrd<T> {
	fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&o.0)
			.map(|r| unsafe { std::mem::transmute(-1i8 * r as i8) } )
	}

	fn lt(&self, o: &Self) -> bool { o.0.lt(&self.0) }
	fn le(&self, o: &Self) -> bool { o.0.le(&self.0) }
	fn gt(&self, o: &Self) -> bool { o.0.gt(&self.0) }
	fn ge(&self, o: &Self) -> bool { o.0.ge(&self.0) }
}

impl<T: Ord> Ord for RevOrd<T> {
	fn cmp(&self, o: &Self) -> Ordering {
		unsafe { std::mem::transmute(-1i8 * self.0.cmp(&o.0) as i8) }
	}

	fn min(self, o: Self) -> Self { RevOrd(self.0.max(o.0)) }
	fn max(self, o: Self) -> Self { RevOrd(self.0.min(o.0)) }
}

impl<T> From<T> for RevOrd<T> { fn from(v: T) -> Self { RevOrd(v) } }

impl<T: ?Sized> Deref for RevOrd<T> {
	type Target = T;
	fn deref(&self) -> &T { &self.0 }
}

impl<T: ?Sized> DerefMut for RevOrd<T> {
	fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}


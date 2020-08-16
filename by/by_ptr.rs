use super::*;

#[derive(Debug, Copy, Clone, Deref, DerefMut)]
pub struct ByPtr<T: ?Sized>(pub T);

impl<T> ByPtr<T> {
	pub fn new(v: T) -> Self { Self(v) }

	pub fn into_inner(self) -> T { self.0 }
}

fn ptr<T: ?Sized + Deref>(x: &ByPtr<T>) -> *const T::Target { (&***x) as _ }

impl<T: ?Sized + Deref> PartialEq for ByPtr<T> {
	fn eq(&self, o: &Self) -> bool { ptr(self) == ptr(o) }
	fn ne(&self, o: &Self) -> bool { ptr(self) != ptr(o) }
}

impl<T: ?Sized + Deref> Eq for ByPtr<T> {}

impl<T: ?Sized + Deref> Hash for ByPtr<T> {
	fn hash<H: Hasher>(&self, hasher: &mut H) { ptr(self).hash(hasher) }
}

impl<T: ?Sized + Deref> PartialOrd for ByPtr<T> {
	fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
		Some(ptr(&self).cmp(&ptr(&o)))
	}

	fn lt(&self, o: &Self) -> bool { ptr(self).lt(&ptr(o)) }
	fn le(&self, o: &Self) -> bool { ptr(self).le(&ptr(o)) }
	fn gt(&self, o: &Self) -> bool { ptr(self).gt(&ptr(o)) }
	fn ge(&self, o: &Self) -> bool { ptr(self).ge(&ptr(o)) }
}

impl<T: ?Sized + Deref> Ord for ByPtr<T> {
	fn cmp(&self, o: &Self) -> Ordering { ptr(self).cmp(&ptr(o)) }
}

impl<T> From<T> for ByPtr<T> { fn from(v: T) -> Self { ByPtr(v) } }

#[cfg(feature = "evmap")]
impl<T> evmap::shallow_copy::ShallowCopy for ByPtr<T> where
	T: evmap::shallow_copy::ShallowCopy
{
	unsafe fn shallow_copy(&mut self) -> Self { Self(self.0.shallow_copy()) }
}


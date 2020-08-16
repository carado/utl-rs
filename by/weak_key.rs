use super::*;

#[derive(Clone)]
pub struct WeakKey<T>(pub T);

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

impl<T> Deref for WeakKey<T> {
	type Target = T;
	fn deref(&self) -> &T { &self.0 }
}

impl<T> DerefMut for WeakKey<T> {
	fn deref_mut(&mut self) -> &mut T { &mut self.0 }
}


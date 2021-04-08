use super::*;

pub unsafe trait VecExt<T>: std::ops::DerefMut<Target = [T]> + ExtendExt<T> {
	fn clear(&mut self);
	fn pop(&mut self) -> Option<T>;
}

unsafe impl<T> VecExt<T> for Vec<T> {
	fn clear(&mut self) { Vec::clear(self); }
	fn pop(&mut self) -> Option<T> { Vec::pop(self) }
}


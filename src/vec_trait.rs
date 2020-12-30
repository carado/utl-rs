use super::*;

pub unsafe trait VecTrait<T>: std::ops::DerefMut<Target = [T]> + ExtendExt<T> {
	fn clear(&mut self);
}

unsafe impl<T> VecTrait<T> for Vec<T> {
	fn clear(&mut self) { Vec::clear(self); }
}


use super::*;

pub unsafe trait VecExt<T>: std::ops::DerefMut<Target = [T]> + ExtendExt<T> {
	fn clear(&mut self);

	fn pop(&mut self) -> Option<T>;

	// retain, but might reorder items
	fn retain_unstable(&mut self, mut retain: impl FnMut(&mut T) -> bool) {
		if self.is_empty() {
			return;
		}

		let mut index = 0;

		while let Some(val) = self.get_mut(index) {
			if retain(val) {
				index += 1;
			} else {
				let last = unsafe { self.pop().unwrap_unchecked() };
				if let Some(val) = self.get_mut(index) {
					*val = last;
				} else {
					break;
				}
			}
		}
	}
}

unsafe impl<T> VecExt<T> for Vec<T> {
	fn clear(&mut self) { Vec::clear(self); }

	fn pop(&mut self) -> Option<T> { Vec::pop(self) }
}


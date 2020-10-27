use std::iter::{TrustedLen};

pub trait ExtendExt<T>: Extend<T> {
	fn extend_copy_slice(&mut self, s: &[T]) where T: Copy;
	fn extend_trusted_len(&mut self, i: impl TrustedLen<Item = T>);
}

impl<T> ExtendExt<T> for Vec<T> {
	fn extend_copy_slice(&mut self, s: &[T]) where T: Copy {
		self.extend_from_slice(s);
	}

	fn extend_trusted_len(&mut self, i: impl TrustedLen<Item = T>) {
		self.extend(i);
	}
}


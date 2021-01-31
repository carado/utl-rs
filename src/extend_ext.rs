use std::iter::{TrustedLen};

pub trait ExtendExt<T>: Extend<T> {
	fn extend_copy_slice(&mut self, s: &[T]) where T: Copy;
	fn extend_trusted_len(&mut self, i: impl TrustedLen<Item = T>);
	fn extend_append_self(&mut self, rhs: &mut Self);
	fn extend_append_vec(&mut self, rhs: &mut Vec<T>);
}

impl<T> ExtendExt<T> for Vec<T> {
	fn extend_copy_slice(&mut self, s: &[T]) where T: Copy {
		self.extend_from_slice(s);
	}

	fn extend_trusted_len(&mut self, i: impl TrustedLen<Item = T>) {
		self.extend(i);
	}

	fn extend_append_self(&mut self, rhs: &mut Self) { self.append(rhs); }

	fn extend_append_vec(&mut self, rhs: &mut Vec<T>) { self.append(rhs); }
}

/*
impl<T: ExtendExt<U>, U> ExtendExt<U> for &'_ mut T {
	fn extend_copy_slice(&mut self, s: &[T]) where T: Copy {
		(**self).extend_copy_slice(s);
	}

	fn extend_trusted_len(&mut self, i: impl TrustedLen<Item = T>) {
		(**self).extend_trusted_len(i);
	}
}
*/


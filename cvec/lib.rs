#![feature(
	allocator_api,
	trusted_len,
)]

use std::{
	ptr::{self, NonNull},
	iter::{IntoIterator, Iterator, TrustedLen},
	mem::{size_of, forget, replace},
	slice,
	ops,
	alloc::{AllocErr, Layout, Global as Alloc, AllocRef},
};

#[repr(C)]
pub struct CVec<T> {
	data: NonNull<T>,
	len: usize,
}

impl<T> CVec<T> {
	pub fn new() -> Self {
		Self { len: 0, data: Alloc.alloc(Self::layout(0)).unwrap().cast() }
	}

	pub fn from_trusted_len(iter: impl TrustedLen<Item = T>) -> Self {
		let mut v = Self::new();
		v.extend_trusted_len(iter);
		v
	}

	pub fn from_copy(iter: &[T]) -> Self where T: Copy {
		let mut v = Self::new();
		v.extend_copy(iter);
		v
	}

	fn len_cap(len: usize) -> usize {
		let cap = match len {
			0 => 0,
			_ => 1 << (size_of::<usize>() as u32 * 8 - (len - 1).leading_zeros()),
		};

		debug_assert!(cap >= len);

		cap
	}

	pub fn cap(&self) -> usize { Self::len_cap(self.len) }

	fn layout(cap: usize) -> Layout {
		debug_assert!(cap.count_ones() <= 1);
		Layout::array::<T>(cap).unwrap()
	}

	unsafe fn dealloc(&self) {
		Alloc.dealloc(self.data.cast(), Self::layout(self.cap()));
	}

	unsafe fn resize_cap(
		&mut self, old_cap: usize, new_cap: usize,
		f: unsafe fn(&mut Alloc, NonNull<u8>, Layout, usize) ->
			Result<NonNull<[u8]>, AllocErr>,
	) {
		debug_assert!(new_cap.count_ones() <= 1 && old_cap.count_ones() <= 1);
		self.data = f(
			&mut Alloc,
			self.data.cast(),
			Self::layout(old_cap),
			size_of::<T>() * new_cap,
		).unwrap().cast();
	}

	unsafe fn grow_len(&mut self, new_len: usize) {
		debug_assert!(new_len >= self.len);

		let old_cap = self.cap();
		let new_cap = Self::len_cap(new_len);

		if new_cap != old_cap {
			self.resize_cap(old_cap, new_cap, Alloc::grow);
		}

		self.len = new_len;
	}

	pub fn extend_trusted_len(&mut self, elems: impl TrustedLen<Item = T>) {
		unsafe {
			let pos = self.len;

			self.grow_len(self.len + elems.size_hint().0);

			for (elem, i) in elems.zip(pos ..) {
				debug_assert!(i < self.len);
				ptr::write(self.data.as_ptr().add(i), elem);
			}
		}
	}

	pub fn extend_copy(&mut self, data: &[T]) where T: Copy {
		unsafe {
			let pos = self.len;

			self.grow_len(self.len + data.len());

			ptr::copy_nonoverlapping(
				data.as_ptr(),
				self.data.as_ptr().add(pos),
				data.len(),
			);
		}
	}

	#[inline]
	pub fn push(&mut self, elem: T) {
		unsafe {
			let old_len = self.len;

			self.len += 1;

			if old_len.count_ones() <= 1 {
				let new_cap = self.cap();
				debug_assert_eq!(new_cap >> 1, Self::len_cap(old_len));
				self.resize_cap(new_cap >> 1, new_cap, Alloc::grow);
			}

			ptr::write(self.data.as_ptr().add(old_len), elem);
		}
	}

	pub fn pop(&mut self) -> Option<T> {
		if self.len > 0 {
			unsafe {
				let new_len = self.len - 1;

				let elem = ptr::read(self.data.as_ptr().add(new_len));

				if new_len.count_ones() <= 1 {
					let old_cap = self.cap();
					debug_assert_eq!(old_cap >> 1, Self::len_cap(new_len));
					self.resize_cap(old_cap, old_cap >> 1, Alloc::shrink);
				}

				self.len = new_len;

				Some(elem)
			}
		} else { None }
	}

	pub fn as_non_null(&self) -> NonNull<T> { self.data }

	pub fn as_ptr(&self) -> *mut T { self.data.as_ptr() }
}

impl<T> IntoIterator for CVec<T> {
	type IntoIter = IntoIter<T>;
	type Item = T;
	fn into_iter(self) -> IntoIter<T> { IntoIter { vec: self, pos: 0 } }
}

pub struct IntoIter<T> { vec: CVec<T>, pos: usize }

impl<T> Drop for IntoIter<T> {
	fn drop(&mut self) {
		while let Some(_) = self.next() {}
		unsafe { self.vec.dealloc(); }
		forget(replace(&mut self.vec, CVec::new()));
	}
}

impl<T> Iterator for IntoIter<T> {
	type Item = T;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let rem = self.vec.len - self.pos;
		(rem, Some(rem))
	}

	fn next(&mut self) -> Option<T> {
		if self.pos == self.vec.len {
			None
		} else {
			let elem = unsafe { ptr::read(self.vec.as_ptr().add(self.pos)) };
			self.pos += 1;
			Some(elem)
		}
	}
}

unsafe impl<T> TrustedLen for IntoIter<T> {}

impl<T> Default for CVec<T> {
	fn default() -> Self { Self::new() }
}

impl<T> Drop for CVec<T> {
	fn drop(&mut self) {
		unsafe {
			for i in 0..self.len {
				ptr::read(self.as_ptr().add(i));
			}

			self.dealloc();
		}
	}
}

impl<T: Clone> Clone for CVec<T> {
	fn clone(&self) -> Self {
		let mut v = Self::new();
		v.extend_trusted_len(self.iter().cloned());
		v
	}
}

impl<T> ops::Deref for CVec<T> {
	type Target = [T];
	fn deref(&self) -> &[T] {
		unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
	}
}

impl<T> ops::DerefMut for CVec<T> {
	fn deref_mut(&mut self) -> &mut [T] {
		unsafe { slice::from_raw_parts_mut(self.as_ptr(), self.len) }
	}
}


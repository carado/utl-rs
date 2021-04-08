use {
	std::{
		ptr::{self, NonNull},
		iter::{IntoIterator, Iterator, TrustedLen, ExactSizeIterator},
		mem::{size_of, forget, replace},
		slice,
		ops,
		alloc::{AllocError, Layout, Global as Alloc, Allocator},
	},
	crate::extend_ext::ExtendExt,
};

#[repr(C)]
pub struct CVec<T> {
	data: NonNull<T>,
	len: usize,
}

impl<T> CVec<T> {
	pub fn new() -> Self {
		Self { len: 0, data: Alloc.allocate(Self::layout(0)).unwrap().cast() }
	}

	pub fn from_trusted_len(iter: impl TrustedLen<Item = T>) -> Self {
		let mut v = Self::new();
		v.extend_trusted_len(iter);
		v
	}

	pub fn from_copy(iter: &[T]) -> Self where T: Copy {
		let mut v = Self::new();
		v.extend_copy_slice(iter);
		v
	}

	pub fn into_raw(self) -> (NonNull<T>, usize) {
		let pair = (self.data, self.len);
		forget(self);
		pair
	}

	pub unsafe fn from_raw(data: NonNull<T>, len: usize) -> Self {
		Self { data, len }
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
		Alloc.deallocate(self.data.cast(), Self::layout(self.cap()));
	}

	unsafe fn resize_cap(
		&mut self, old_cap: usize, new_cap: usize,
		f: unsafe fn(&Alloc, NonNull<u8>, Layout, Layout) ->
			Result<NonNull<[u8]>, AllocError>,
	) {
		debug_assert!(new_cap.count_ones() <= 1 && old_cap.count_ones() <= 1);
		self.data = f(
			&mut Alloc,
			self.data.cast(),
			Self::layout(old_cap),
			Self::layout(new_cap),
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

	pub fn resize_with(&mut self, to: usize, mut with: impl FnMut() -> T) {
		unsafe {
			let range = self.len() .. to;

			use std::cmp::Ordering::*;

			if let Some(resize) = match to.cmp(&self.len()) {
				Less => Some(Alloc::shrink as _),
				Greater => Some(Alloc::grow as _),
				Equal => None,
			} {
				self.resize_cap(Self::len_cap(self.len()), Self::len_cap(to), resize);
			}

			for i in range {
				self.data.as_ptr().add(i).write(with());
			}
		}
	}

	pub fn resize(&mut self, to: usize, value: T) where T: Clone {
		self.resize_with(to, || value.clone());
	}

	pub fn as_non_null(&self) -> NonNull<T> { self.data }

	pub fn as_ptr(&self) -> *mut T { self.data.as_ptr() }

	pub fn reserve(&self, _: usize) {} // intentionally a no-op

	// for now just RangeFull (`..`)
	pub fn drain(&mut self, _: std::ops::RangeFull) -> IntoIter<T> {
		std::mem::replace(self, Self::new()).into_iter()
	}

	pub fn clear(&mut self) {
		*self = Self::new();
	}

	unsafe fn extend_copy_unsafe(&mut self, data: &[T]) {
		let pos = self.len;

		self.grow_len(self.len + data.len());

		ptr::copy_nonoverlapping(
			data.as_ptr(),
			self.data.as_ptr().add(pos),
			data.len(),
		);
	}

	pub fn clear_forget(&mut self) {
		unsafe { self.dealloc(); }
		forget(replace(self, Self::new()));
	}
}

impl<T> Extend<T> for CVec<T> {
	fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
		unsafe {
			let iter = iter.into_iter();

			let mut alloc_cap = Self::len_cap(self.len + iter.size_hint().0);

			self.resize_cap(self.cap(), alloc_cap, Alloc::grow);

			for (item, pos) in iter.zip(self.len ..) {
				if pos == alloc_cap {
					let new_alloc_cap = (alloc_cap * 2).max(1);
					self.resize_cap(alloc_cap, new_alloc_cap, Alloc::grow);
					alloc_cap = new_alloc_cap;
				}

				self.data.as_ptr().add(pos).write(item);
				self.len += 1;
			}

			let should_cap = Self::len_cap(self.len);
			if should_cap != alloc_cap {
				debug_assert!(should_cap < alloc_cap);
				self.resize_cap(alloc_cap, should_cap, Alloc::shrink);
			}
		}
	}

	fn extend_one(&mut self, val: T) { self.push(val); }
}

impl<T> ExtendExt<T> for CVec<T> {
	fn extend_trusted_len(&mut self, elems: impl TrustedLen<Item = T>) {
		unsafe {
			let pos = self.len;

			self.grow_len(self.len + elems.size_hint().1.unwrap());

			for (elem, i) in elems.zip(pos ..) {
				debug_assert!(i < self.len);
				ptr::write(self.data.as_ptr().add(i), elem);
			}
		}
	}

	fn extend_copy_slice(&mut self, data: &[T]) where T: Copy {
		unsafe { self.extend_copy_unsafe(data); }
	}

	fn extend_append_self(&mut self, rhs: &mut Self) {
		self.extend_append_cvec(rhs);
	}

	fn extend_append_cvec(&mut self, rhs: &mut CVec<T>) {
		if self.is_empty() {
			std::mem::swap(self, rhs);
		} else {
			unsafe {
				self.extend_copy_unsafe(&**rhs);
				rhs.clear_forget();
			}
		}
	}

	fn extend_append_vec(&mut self, rhs: &mut Vec<T>) {
		unsafe {
			self.extend_copy_unsafe(&**rhs);
			rhs.set_len(0);
		}
	}
}

impl<T> IntoIterator for CVec<T> {
	type IntoIter = IntoIter<T>;
	type Item = T;
	fn into_iter(self) -> IntoIter<T> { IntoIter { vec: self, pos: 0 } }
}

impl<'a, T> IntoIterator for &'a CVec<T> {
	type IntoIter = std::slice::Iter<'a, T>;
	type Item = &'a T;
	fn into_iter(self) -> Self::IntoIter { (**self).iter() }
}

impl<'a, T> IntoIterator for &'a mut CVec<T> {
	type IntoIter = std::slice::IterMut<'a, T>;
	type Item = &'a mut T;
	fn into_iter(self) -> Self::IntoIter { (**self).iter_mut() }
}

pub struct IntoIter<T> { vec: CVec<T>, pos: usize }

impl<T> Drop for IntoIter<T> {
	fn drop(&mut self) {
		while let Some(_) = self.next() {}
		self.vec.clear_forget();
	}
}

impl<T> Iterator for IntoIter<T> {
	type Item = T;

	fn size_hint(&self) -> (usize, Option<usize>) {
		let len = self.len();
		(len, Some(len))
	}

	fn next(&mut self) -> Option<T> {
		if self.is_empty() {
			None
		} else {
			let elem = unsafe { ptr::read(self.vec.as_ptr().add(self.pos)) };
			self.pos += 1;
			Some(elem)
		}
	}
}

impl<T> ExactSizeIterator for IntoIter<T> {
	fn len(&self) -> usize { self.vec.len - self.pos }
	fn is_empty(&self) -> bool { self.vec.len == self.pos }
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

unsafe impl<T: Send> Send for CVec<T> {}
unsafe impl<T: Sync> Sync for CVec<T> {}

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

impl<T: std::fmt::Debug> std::fmt::Debug for CVec<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) ->
		Result<(), std::fmt::Error>
	{
		(**self).fmt(f)
	}
}

impl<T: PartialEq> PartialEq for CVec<T> {
	fn eq(&self, rhs: &Self) -> bool { **self == **rhs }
	fn ne(&self, rhs: &Self) -> bool { **self != **rhs }
}

impl<T: Eq> Eq for CVec<T> {}

impl<T: std::hash::Hash> std::hash::Hash for CVec<T> {
	fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
		(**self).hash(h);
	}
}

impl<T: PartialOrd> PartialOrd for CVec<T> {
	fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
		(**self).partial_cmp(&**rhs)
	}

	fn lt(&self, rhs: &Self) -> bool { **self <  **rhs }
	fn le(&self, rhs: &Self) -> bool { **self <= **rhs }
	fn gt(&self, rhs: &Self) -> bool { **self >  **rhs }
	fn ge(&self, rhs: &Self) -> bool { **self >= **rhs }
}

impl<T: Ord> Ord for CVec<T> {
	fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
		(**self).cmp(&**rhs)
	}
}

unsafe impl<T> super::vec_ext::VecExt<T> for CVec<T> {
	fn clear(&mut self) { CVec::clear(self); }
	fn pop(&mut self) -> Option<T> { CVec::pop(self) }
}

#[test]
fn test() {
	let mut vec;

	eprintln!("correct size_hint");
	vec = CVec::new();
	vec.extend(0..1000);

	eprintln!("too small size_hint");
	vec = CVec::new();
	vec.extend((0..100).chain((0..1000).filter(|_| true)));

	eprintln!("too large size_hint");

	struct I(std::ops::Range<i32>);

	impl Iterator for I {
		type Item = i32;
		fn next(&mut self) -> Option<i32> { self.0.next() }
		fn size_hint(&self) -> (usize, Option<usize>) { (1000, None) }
	}

	vec = CVec::new();
	vec.extend(I(0..100));
}


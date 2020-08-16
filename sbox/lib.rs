#![feature(
	raw,
	unsize,
	untagged_unions,
)]

use std::{*, raw::TraitObject, mem::ManuallyDrop};

#[repr(C)]
pub struct SBox<T: ?Sized, S = [usize; 1]> {
	vtable: num::NonZeroUsize,
	data: S,
	_phantom: marker::PhantomData<T>,
}

#[repr(C)]
pub union SBoxOr<T: ?Sized, U, S = [usize; 1]> {
	sbox: ManuallyDrop<SBox<T, S>>,
	or: ManuallyDrop<Or<U>>,
}

#[repr(C)]
struct Or<U> { zero: usize, value: U }

pub fn sbox<T, U, S>(value: U) -> SBox<T, S> where
	T: ?Sized + marker::Unsize<T>,
	U: marker::Unsize<T>,
	Box<U>: marker::Unsize<T>,
{
	SBox::new(value)
}

impl<T: ?Sized, S> SBox<T, S> {
	pub fn new<U: marker::Unsize<T>>(val: U) -> Self where
		Box<U>: marker::Unsize<T>
	{
		if
			mem::align_of::<U>() > mem::align_of::<S>() ||
			mem::size_of ::<U>() > mem::size_of ::<S>()
		{
			Self::new_fitting(Box::new(val))
		} else {
			Self::new_fitting(val)
		}
	}

	pub fn new_fitting<U: marker::Unsize<T>>(val: U) -> Self {
		assert!(mem::size_of ::< U>() <= mem::size_of ::<S>());
		assert!(mem::align_of::< U>() <= mem::align_of::<S>());
		assert!(mem::size_of ::<&T>() == mem::size_of::<TraitObject>());

		unsafe {
			let vtable = mem::transmute_copy::<&T, TraitObject>(&(&val as &T)).vtable;
			let mut data = mem::MaybeUninit::<S>::zeroed(); //TODO try out uninit()
			ptr::copy::<U>(&val as *const _, data.as_mut_ptr() as *mut _, 1);
			mem::forget(val);

			Self {
				vtable: num::NonZeroUsize::new(vtable as _).expect("zero vtable"),
				data: data.assume_init(),
				_phantom: marker::PhantomData,
			}
		}
	}
}

impl<T: ?Sized, U, S> SBoxOr<T, U, S> {
	pub fn new_sbox(sbox: SBox<T, S>) -> Self {
		Self { sbox: ManuallyDrop::new(sbox) }
	}

	pub fn new_or(or: U) -> Self {
		Self { or: ManuallyDrop::new(Or { zero: 0, value: or }) }
	}

	pub fn is_sbox(&self) -> bool { unsafe { self.or.zero != 0 } }
	pub fn is_or(&self) -> bool { unsafe { self.or.zero == 0 } }

	pub fn sbox(&self) -> Result<&SBox<T, S>, &U> {
		unsafe { if self.is_sbox() { Ok(&self.sbox) } else { Err(&self.or.value) } }
	}

	pub fn or(&self) -> Result<&U, &SBox<T, S>> {
		unsafe { if self.is_or() { Ok(&self.or.value) } else { Err(&self.sbox) } }
	}

	pub fn sbox_mut(&mut self) -> Result<&mut SBox<T, S>, &mut U> { unsafe {
		if self.is_sbox() { Ok(&mut self.sbox) } else { Err(&mut self.or.value) }
	} }

	pub fn or_mut(&mut self) -> Result<&mut U, &mut SBox<T, S>> { unsafe {
		if self.is_or() { Ok(&mut self.or.value) } else { Err(&mut self.sbox) }
	} }

	pub fn into_sbox(mut self) -> Result<SBox<T, S>, U> { unsafe {
		if self.is_sbox() {
			Ok(ManuallyDrop::take(&mut self.sbox))
		} else {
			Err(ManuallyDrop::take(&mut self.or).value)
		}
	} }

	pub fn into_or(mut self) -> Result<U, SBox<T, S>> { unsafe {
		if self.is_or() {
			Ok(ManuallyDrop::take(&mut self.or).value)
		} else {
			Err(ManuallyDrop::take(&mut self.sbox))
		}
	} }
}

impl<T: ?Sized, U, S> Drop for SBoxOr<T, U, S> {
	fn drop(&mut self) { unsafe {
		if self.is_sbox() {
			ManuallyDrop::drop(&mut self.sbox);
		} else {
			ManuallyDrop::drop(&mut self.or);
		}
	} }
}

impl<T: ?Sized, S> ops::Deref for SBox<T, S> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe {
			assert!(mem::size_of::<&T>() == mem::size_of::<TraitObject>());
			mem::transmute_copy::<TraitObject, &T>(&TraitObject {
				vtable: self.vtable.get() as _,
				data: &self.data as *const _ as _,
			})
		}
	}
}

impl<T: ?Sized, S> ops::DerefMut for SBox<T, S> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe {
			assert!(mem::size_of::<&mut T>() == mem::size_of::<TraitObject>());
			mem::transmute_copy::<TraitObject, &mut T>(&TraitObject {
				vtable: self.vtable.get() as _,
				data: &mut self.data as *mut _ as _,
			})
		}
	}
}

impl<T: ?Sized, S> Drop for SBox<T, S> {
	fn drop(&mut self) {
		unsafe {
			assert!(
				mem::size_of::<&mut mem::ManuallyDrop<T>>() ==
				mem::size_of::<TraitObject>()
			);

			mem::ManuallyDrop::drop(
				mem::transmute_copy::<_, &mut mem::ManuallyDrop<T>>(&TraitObject {
					vtable: self.vtable.get() as _,
					data: &mut self.data as *mut _ as _,
				})
			);
		}
	}
}

impl<T: ?Sized + PartialEq, S> PartialEq for SBox<T, S> {
	fn eq(&self, rhs: &Self) -> bool { (**self).eq(rhs) }
	fn ne(&self, rhs: &Self) -> bool { (**self).ne(rhs) }
}

impl<T: ?Sized + Eq, S> Eq for SBox<T, S> {}

impl<T: ?Sized + hash::Hash, S> hash::Hash for SBox<T, S> {
	fn hash<H: hash::Hasher>(&self, h: &mut H) {
		self.vtable.hash(h);
		(**self).hash(h);
	}
}

unsafe impl<T: ?Sized + Send, S> Send for SBox<T, S> {}
unsafe impl<T: ?Sized + Sync, S> Sync for SBox<T, S> {}

impl<T: ?Sized + PartialEq, U: PartialEq, S> PartialEq for SBoxOr<T, U, S> {
	fn eq(&self, rhs: &Self) -> bool { self.sbox() == rhs.sbox() }
	fn ne(&self, rhs: &Self) -> bool { self.sbox() != rhs.sbox() }
}

impl<T: ?Sized + Eq, U: Eq, S> Eq for SBoxOr<T, U, S> {}

impl<T: ?Sized + hash::Hash, U: hash::Hash, S> hash::Hash for SBoxOr<T, U, S> {
	fn hash<H: hash::Hasher>(&self, h: &mut H) {
		match self.sbox() { Ok(v) => v.hash(h), Err(v) => v.hash(h) }
	}
}

unsafe impl<T: ?Sized + Send, U: Send, S> Send for SBoxOr<T, U, S> {}
unsafe impl<T: ?Sized + Sync, U: Sync, S> Sync for SBoxOr<T, U, S> {}


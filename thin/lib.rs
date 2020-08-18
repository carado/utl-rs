#![feature(
	raw,
	unsize,
	allocator_api,
	slice_ptr_get,
)]

use std::{
	num::NonZeroUsize,
	marker::{Unsize, PhantomData},
	mem::{size_of, transmute_copy, forget},
	ptr::{self, NonNull},
	raw::TraitObject,
	alloc::{AllocRef, Global as Alloc, Layout},
	ops::{Deref, DerefMut},
	any::Any,
};

pub struct Thin<T: ?Sized, Al = usize>(NonZeroUsize, PhantomData<(Al, T)>);

impl<T: ?Sized, Al> Thin<T, Al> {
	pub fn new<U: Unsize<T>>(value: U) -> Self {
		unsafe {
			assert_eq!(size_of::<&T>(), 2 * size_of::<usize>());

			let (layout, offset) = Layout::new::<*mut ()>()
				.extend(Layout::new::<U>()).unwrap();

			assert_eq!(offset, size_of::<Al>());

			let mem = Alloc.alloc(layout).unwrap().as_mut_ptr();

			ptr::write(
				mem as _,
				transmute_copy::<&T, TraitObject>(&(&value as &T)).vtable,
			);

			ptr::write(mem.add(offset) as _, value);

			Self(NonZeroUsize::new_unchecked(mem as usize), PhantomData)
		}
	}

	pub fn as_dyn_ptr(self_: &Self) -> *const T {
		unsafe {
			let ptr = self_.0.get() as *const u8;
			transmute_copy::<TraitObject, &T>(&TraitObject {
				vtable: *(ptr as *const *mut ()),
				data: ptr.add(size_of::<Al>()) as _,
			})
		}
	}

	pub fn as_u8_ptr(self_: &Self) -> *const u8 { self_.0.get() as _ }

	pub fn into_u8_ptr(self_: Self) -> *const u8 {
		let ptr = Self::as_u8_ptr(&self_);
		forget(self_);
		ptr
	}

	pub unsafe fn from_u8_ptr(ptr: *const u8) -> Self {
		Self(NonZeroUsize::new(ptr as _).unwrap(), PhantomData)
	}

	/*
	pub fn as_dyn_non_null(self_: &Self) -> NonNull<T> {
		unsafe { NonNull::new_unchecked(Self::as_dyn_ptr(self_) as _) }
	}

	pub fn as_u8_non_null(self_: &Self) -> NonNull<u8> {
		unsafe { NonNull::new_unchecked(Self::as_u8_ptr(self_) as _) }
	}

	pub fn into_u8_non_null(self_: Self) -> NonNull<u8> {
		unsafe { NonNull::new_unchecked(Self::into_u8_ptr(self_) as _) }
	}

	pub unsafe fn from_u8_non_null(ptr: NonNull<u8>) -> Self {
		Self(NonZeroUsize::new_unchecked(ptr.as_ptr() as _), PhantomData)
	}
	*/

	unsafe fn deallocer(next: Layout) -> impl FnOnce(*const u8) {
		let (layout, offset) = Layout::new::<*mut ()>().extend(next).unwrap();
		assert_eq!(offset, size_of::<Al>());
		move |ptr| Alloc.dealloc(NonNull::new_unchecked(ptr as _), layout)
	}

	pub unsafe fn deref_from_u8_ptr(ptr: *const u8) -> *const T {
		let self_ = Self::from_u8_ptr(ptr);
		let dyn_ = Self::as_dyn_ptr(&self_);
		forget(self_);
		dyn_
	}
}

impl<T: ?Sized, Al> Deref for Thin<T, Al> {
	type Target = T;
	fn deref(&self) -> &T {
		unsafe { &*Self::as_dyn_ptr(self) }
	}
}

impl<T: ?Sized, Al> DerefMut for Thin<T, Al> {
	fn deref_mut(&mut self) -> &mut T {
		unsafe { &mut *(Self::as_dyn_ptr(self) as *mut _) }
	}
}

impl<T: ?Sized, Al> Drop for Thin<T, Al> {
	fn drop(&mut self) {
		unsafe {
			let ptr = Self::as_dyn_ptr(self) as *mut T;
			let dealloc = Self::deallocer(Layout::for_value(&*ptr));
			ptr::drop_in_place(ptr);
			dealloc(Self::as_u8_ptr(&self));
		}
	}
}

impl<Al> Thin<dyn Any, Al> {
	pub fn downcast<U: 'static>(self_: Self) -> Result<U, Self> {
		if self_.is::<U>() {
			unsafe {
				let val = ptr::read(Self::as_u8_ptr(&self_).add(size_of::<Al>()) as _);
				Self::deallocer(Layout::new::<U>())(Self::as_u8_ptr(&self_) as _);
				forget(self_);
				Ok(val)
			}
		} else {
			Err(self_)
		}
	}
}

unsafe impl<T: ?Sized + Send, Al> Send for Thin<T, Al> {}
unsafe impl<T: ?Sized + Sync, Al> Sync for Thin<T, Al> {}


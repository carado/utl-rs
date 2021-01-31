pub fn mutate_get<T, R>(ref_: &mut T, func: impl FnOnce(T) -> (T, R)) -> R {
	use std::mem::{ManuallyDrop, MaybeUninit};

	let mut ret: MaybeUninit<R> = MaybeUninit::uninit();

	struct ApplyOnDrop<'a, T, R, F: FnOnce(T) -> (T, R)> {
		ref_: &'a mut T,
		ret: &'a mut MaybeUninit<R>,
		func: ManuallyDrop<F>,
	}

	impl<'a, T, R, F: FnOnce(T) -> (T, R)> Drop for ApplyOnDrop<'a, T, R, F> {
		fn drop(&mut self) {
			unsafe {
				let value = (self.ref_ as *mut T).read();
				// at this time there are two T's on the stack when there should be only
				// one. this would be a problem if the stack were to unwind; however,
				// because we are inside Drop::drop, any panic is guaranteed to abort.
				let (new, ret) = ManuallyDrop::take(&mut self.func)(value);
				(self.ref_ as *mut T).write(new);
				self.ret.write(ret);
			}
		}
	}

	drop(ApplyOnDrop { ref_, ret: &mut ret, func: ManuallyDrop::new(func) });

	unsafe { ret.assume_init() }
}

pub fn mutate<T>(ref_: &mut T, func: impl FnOnce(T) -> T) {
	mutate_get(ref_, move |v| (func(v), ()))
}


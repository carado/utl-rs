pub use std::{
	hint::spin_loop as spin_loop_hint,

	sync::atomic::{
		AtomicBool as bool,
		AtomicU8 as u8, AtomicI8 as i8,
		AtomicU16 as u16, AtomicI16 as i16,
		AtomicU32 as u32, AtomicI32 as i32,
		AtomicU64 as u64, AtomicI64 as i64,
		AtomicUsize as usize, AtomicIsize as isize,
		AtomicPtr as Ptr,
		Ordering,
		fence, compiler_fence,
		Ordering::*,
	},
};


#![feature(
	coerce_unsized,
	unsize,
)]

use {
	std::{
		ops::{Deref, DerefMut, CoerceUnsized},
		hash::{Hash, Hasher},
		borrow::*,
		cmp::Ordering,
	},
	derive_more::{Deref, DerefMut, Constructor},
};

mod by_ptr;
mod by_key;
mod weak_key;
mod bits;

pub use {by_ptr::*, by_key::*, weak_key::*, bits::*};


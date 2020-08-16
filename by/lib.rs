use {
	std::{ops::{Deref, DerefMut}, hash::{Hash, Hasher}, borrow::*, cmp::Ordering},
	derive_more::{Deref, DerefMut, Constructor},
};

mod by_ptr;
mod by_key;
mod weak_key;

pub use {by_ptr::*, by_key::*, weak_key::*};


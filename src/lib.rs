#![feature(
	raw,
	coerce_unsized,
	unsize,
	allocator_api,
	trait_alias,
	extend_one,
	trusted_len,
	slice_ptr_get,
	untagged_unions,
	hash_raw_entry,
	maybe_uninit_extra,
	maybe_uninit_ref,
	bool_to_option,
	exact_size_is_empty,
	generator_trait,
	never_type,
	half_open_range_patterns,
	exclusive_range_pattern,
)]

#![deny(unused_must_use)]

pub mod cvec;
pub mod sbox;
pub mod entry_ext;
pub mod just_hash;
pub mod by_bits;
pub mod by_key;
pub mod weak_key;
pub mod atomic;
pub mod bits_ext;
pub mod on_drop;
pub mod rev_ord;
pub mod maps;
pub mod range_ext;
pub mod thin;
pub mod unreach;
//pub mod alloc_vec;
//pub mod dedup_vec;
pub mod derefs;
pub mod coffer;
pub mod by_ptr;
pub mod non_max;
pub mod extend_ext;
pub mod unsafe_cell;
pub mod generator_state_ext;
pub mod vec_trait;
#[cfg(feature = "serde")] pub mod bytes;
#[cfg(feature = "serde")] pub mod ser_iter;

pub use crate::{
	cvec::CVec,
	sbox::SBox,
	entry_ext::*,
	just_hash::*,
	by_bits::*,
	weak_key::*,
	by_key::*,
	bits_ext::BitsExt,
	on_drop::*,
	rev_ord::*,
	range_ext::*,
	thin::*,
	unreach::*,
	derefs::*,
	by_ptr::*,
	non_max::*,
	extend_ext::*,
	unsafe_cell::{UnsafeCell, UnsafeCellRef, UnsafeCellMut},
	generator_state_ext::*,
	vec_trait::*,
};

pub use ::servo_arc::Arc as SArc;

pub use parking_lot;

pub use num_traits;


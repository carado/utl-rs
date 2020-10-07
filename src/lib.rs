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
pub mod alloc_vec;
pub mod dedup_vec;
pub mod derefs;
pub mod coffer;

pub use crate::{
	cvec::CVec,
	sbox::SBox,
	entry_ext::*,
	just_hash::*,
	by_bits::*,
	weak_key::*,
	by_key::*,
	bits_ext::*,
	on_drop::*,
	rev_ord::*,
	range_ext::*,
	thin::*,
	unreach::*,
	derefs::*,
};

pub use parking_lot;


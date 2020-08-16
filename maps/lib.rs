pub mod no_op;

#[cfg(fnv)]
pub mod shrt {
	pub use fnv::{
		FnvHasher as Hasher,
		FnvBuildHasher as BuildHasher,
		FnvHashMap as Map,
		FnvHashSet as Set,
	};

	pub fn hash(val: &impl super::Hash) -> u64 { super::hash::<Hasher, _>(val) }
}

#[cfg(rustc_hash)]
pub mod int {
	pub use rustc_hash::{
		FxHasher as Hasher,
		FxHashMap as Map,
		FxHashSet as Set,
	};

	pub type BuildHasher = std::hash::BuildHasherDefault<Hasher>;

	pub fn hash(val: &impl super::Hash) -> u64 { super::hash::<Hasher, _>(val) }
}

pub mod std {
	pub use std::collections::{
		HashMap as Map,
		HashSet as Set,
		hash_map::{
			RandomState as BuildHasher,
			DefaultHasher as Hasher,
		},
	};

	pub fn hash(val: &impl super::Hash) -> u64 { super::hash::<Hasher, _>(val) }
}

pub mod det {
	use std::collections::*;

	pub type Hasher = hash_map::DefaultHasher;
	pub type BuildHasher = std::hash::BuildHasherDefault<Hasher>;
	pub type Map<K, V> = HashMap<K, V, BuildHasher>;
	pub type Set<T> = HashSet<T, BuildHasher>;

	pub fn hash(val: &impl super::Hash) -> u64 { super::hash::<Hasher, _>(val) }
}

use ::std::hash::{Hash, Hasher};

fn hash<H: Hasher + Default, T: Hash>(val: &T) -> u64 {
	let mut hasher = H::default();
	val.hash(&mut hasher);
	hasher.finish()
}


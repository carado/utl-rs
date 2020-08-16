use std::hash::{Hash, Hasher, BuildHasher};

pub trait JustHash: BuildHasher {
	fn just_hash(&self, value: &(impl Hash + ?Sized)) -> u64 {
		let mut hasher = self.build_hasher();
		value.hash(&mut hasher);
		hasher.finish()
	}
}

impl<T: BuildHasher> JustHash for T {}


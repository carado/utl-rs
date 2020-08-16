use super::*;

#[derive(Default, Clone, Copy, Debug, Constructor)]
pub struct ByKey<K, V> {
	pub key: K,
	pub val: V,
}

impl<K: PartialEq, V> PartialEq for ByKey<K, V> {
	fn eq(&self, rhs: &Self) -> bool { self.key == rhs.key }
	fn ne(&self, rhs: &Self) -> bool { self.key != rhs.key }
}

impl<K: Eq, V> Eq for ByKey<K, V> {}

impl<K: PartialOrd, V> PartialOrd for ByKey<K, V> {
	fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
		self.key.partial_cmp(&rhs.key)
	}

	fn le(&self, rhs: &Self) -> bool { self.key.le(&rhs.key) }
	fn lt(&self, rhs: &Self) -> bool { self.key.lt(&rhs.key) }
	fn ge(&self, rhs: &Self) -> bool { self.key.ge(&rhs.key) }
	fn gt(&self, rhs: &Self) -> bool { self.key.gt(&rhs.key) }
}

impl<K: Ord, V> Ord for ByKey<K, V> {
	fn cmp(&self, rhs: &Self) -> Ordering { self.key.cmp(&rhs.key) }
}

impl<K: Hash, V> Hash for ByKey<K, V> {
	fn hash<H: Hasher>(&self, hasher: &mut H) {
		self.key.hash(hasher);
	}
}

impl<K, V> Borrow<K> for ByKey<K, V> {
	fn borrow(&self) -> &K { &self.key }
}

impl<K, V> Deref for ByKey<K, V> {
	type Target = V;
	fn deref(&self) -> &V { &self.val }
}

impl<K, V> DerefMut for ByKey<K, V> {
	fn deref_mut(&mut self) -> &mut V { &mut self.val }
}


use std::{
	cmp::Ordering,
	hash::{Hash, Hasher},
	ops::{Deref, DerefMut},
	borrow::{Borrow, BorrowMut},
};

#[derive(Default, Clone, Copy, Debug)]
pub struct ByKey<K, V> {
	pub key: K,
	pub value: V,
}

impl<K, V> ByKey<K, V> {
	pub fn new(key: K, value: V) -> Self { Self { key, value } }
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

impl<K, V> BorrowMut<K> for ByKey<K, V> {
	fn borrow_mut(&mut self) -> &mut K { &mut self.key }
}

impl<K, V> Deref for ByKey<K, V> {
	type Target = V;
	fn deref(&self) -> &V { &self.value }
}

impl<K, V> DerefMut for ByKey<K, V> {
	fn deref_mut(&mut self) -> &mut V { &mut self.value }
}

/*
#[cfg(feature = "serde")]
impl<K: serde::Serialize, V> serde::Serialize for ByKey<K, V> {
	fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
		self.key.serialize(ser)
	}
}

#[cfg(feature = "serde")]
impl<'de, K, V> serde::Deserialize<'de> for ByKey<K, V> where
	K: serde::Deserialize<'de>,
	V: Default,
{
	fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		Ok(Self {
			key: serde::Deserialize::deserialize(de)?,
			value: <_>::default(),
		})
	}
}
*/


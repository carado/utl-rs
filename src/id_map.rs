use {
	super::{*, alloc_vec::AllocVec},
	std::{
		hash::{Hash, BuildHasher},
		ops::{Index, IndexMut},
		collections::HashMap,
		num::NonZeroUsize,
	},
};

type EntryHash = u32;

pub struct IDMap<K, V, S = maps::std::BuildHasher> {
	mapped: HashMap<ByKey<K, usize>, (), S>,
	entries: AllocVec<
		Option<std::cell::UnsafeCell<Entry<V>>>,
		std::collections::BinaryHeap<RevOrd<usize>>,
	>,
}

struct Entry<V> { usage: NonZeroUsize, hash: u64, value: V }

impl<K: Hash + Eq, V, S: Default> Default for IDMap<K, V, S> {
	fn default() -> Self {
		Self {
			mapped: <_>::default(),
			entries: <_>::default(),
		}
	}
}

impl<K: Hash + Eq, V, S: BuildHasher> IDMap<K, V, S> {
	pub fn new() -> Self where S: Default { Self::default() }

	pub fn with_hasher(hasher: S) -> Self {
		Self {
			mapped: HashMap::with_hasher(hasher),
			entries: <_>::default(),
		}
	}
	
	pub fn insert<Q: maps::BorrowKey<K>>(
		&mut self, pre_key: Q, make_value: impl FnOnce(Q) -> (K, V),
	) -> (usize, &mut K, &mut V) {
		let entries = &mut self.entries;

		let hash = self.mapped.hasher().just_hash(&pre_key);

		let (by_key, ()) = self.mapped.raw_entry_mut()
			.from_hash(hash, |k| k.key.borrow() == &pre_key)
			.and_modify(|k, ()| unsafe {
				let entry = &mut *entries
					.get_unchecked_mut(k.value)
					.unsafe_unwrap_mut()
					.get();

				entry.usage = NonZeroUsize::new_unchecked(entry
					.usage.get()
					.checked_add(1)
					.expect("usage overflow")
				);
			})
			.or_insert_with(|| {
				let (key, value) = make_value(pre_key);
				let usage = unsafe { NonZeroUsize::new_unchecked(1) };
				let id = entries.alloc(Some(Entry { usage, hash, value }.into()));
				(ByKey::new(key, id), ())
			});

		let value = unsafe {
			&mut (&mut *entries
				.get_unchecked_mut(by_key.value)
				.unsafe_unwrap_mut()
				.get()
			).value
		};

		(by_key.value, &mut by_key.key, value)
	}

	pub fn get<Q: maps::BorrowKey<K>>(&self, pre_key: &Q) ->
		Option<(usize, &K, &V)>
	{
		let hash = self.mapped.hasher().just_hash(pre_key);

		self.mapped
			.raw_entry()
			.from_hash(hash, |by| by.key.borrow() == pre_key)
			.map(|(ByKey { key, value: id }, _)| unsafe {
				let entry = &*self.entries.get_unchecked(*id).unsafe_unwrap_ref().get();
				(*id, key, &entry.value)
			})
	}

	pub fn remove(&mut self, id: usize) -> Option<V> {
		unsafe {
			let entry = self.entries.get_mut(id)?.take()?.into_inner();

			self.mapped
				.raw_entry_mut()
				.from_hash(entry.hash.as_(), |by| by.value == id)
				.occupied()
				.unsafe_unwrap();

			Some(entry.value)
		}
	}

	pub unsafe fn get_unchecked(&self, id: usize) -> &V {
		&(&*self.entries.get_unchecked(id).unsafe_unwrap_ref().get()).value
	}

	pub unsafe fn get_unchecked_mut(&mut self, id: usize) -> &mut V {
		&mut
			(&mut *self.entries.get_unchecked_mut(id).unsafe_unwrap_mut().get()).value
	}

	pub fn usage(&self, id: usize) -> usize {
		self.entries
			.get(id)
			.and_then(|o| o.as_ref())
			.map_or(0, |e| unsafe { &*e.get() }.usage.get())
	}

	pub fn key_entries(&self) -> impl Iterator<Item = (usize, &K, &V)> {
		self.mapped
			.keys()
			.map(move |by| unsafe {
				let id = by.value;
				(id, &by.key, self.get_unchecked(id))
			})
	}

	pub fn key_entries_mut(&mut self) ->
		impl Iterator<Item = (usize, &K, &mut V)>
	{
		let entries = &mut self.entries;

		self.mapped
			.keys()
			.map(move |by| unsafe {
				let id = by.value;
				let ent = &mut *entries.get_unchecked(id).unsafe_unwrap_ref().get();
				(id, &by.key, &mut ent.value)
			})
	}

	//pub fn at(&self, id: U::Raw) -> Option<(&K, &V)> {
	//pub fn at_mut(&mut self, id: U::Raw) -> Option<(&K, &mut V)> {

	pub fn entries<'a>(&'a self) ->
		impl Iterator<Item = (usize, &'a V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.raw().iter()
			.zip(0..)
			.filter_map(move |(e, id)| unsafe {
				let entry = &*e.as_ref()?.get();
				let key = key_fn(id, entry, mapped);
				Some((id, &entry.value, key))
			})
	}

	pub fn entries_mut<'a>(&'a mut self) ->
		impl Iterator<Item = (usize, &'a mut V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.raw_mut().iter_mut()
			.zip(0..)
			.filter_map(move |(e, id)| unsafe {
				let entry = &mut *e.as_ref()?.get();
				let key = key_fn(id, entry, mapped);
				Some((id, &mut entry.value, key))
			})
	}
}

unsafe fn key_fn<'a, K, V, S: BuildHasher>(
	id: usize, entry: &Entry<V>, mapped: &'a HashMap<ByKey<K, usize>, (), S>,
) -> impl FnOnce() -> &'a K {
	let hash = entry.hash;

	move || &mapped
		.raw_entry()
		.from_hash(hash, |by| by.value == id)
		.unsafe_unwrap()
		.0
		.key
}

impl<K, V, S> Index<usize> for IDMap<K, V, S> {
	type Output = V;
	fn index(&self, id: usize) -> &V {
		&unsafe { &*self.entries[id].as_ref().unwrap().get() }.value
	}
}

impl<K, V, S> IndexMut<usize> for IDMap<K, V, S> {
	fn index_mut(&mut self, id: usize) -> &mut V {
		&mut unsafe { &mut *self.entries[id].as_mut().unwrap().get() }.value
	}
}

#[test]
fn test() {
	use std::mem::size_of;
	assert_eq!(size_of::<Entry<() >>(), size_of::<Option<Entry<() >>>());
	assert_eq!(size_of::<Entry<u32>>(), size_of::<Option<Entry<u32>>>());
}


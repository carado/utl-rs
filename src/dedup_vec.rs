use {
	super::{*, alloc_vec::{AllocVec, FreeSetTrusted}},
	std::{
		hash::{Hash, BuildHasher},
		ops::{Index, IndexMut},
		collections::HashMap,
		num::NonZeroUsize,
	},
	num_traits::NumCast,
};

type EntryHash = u32;

pub struct DedupVec<
	K,
	V,
	F: FreeSetTrusted = std::collections::BinaryHeap<RevOrd<usize>>,
	S = maps::std::BuildHasher,
> {
	mapped: HashMap<ByKey<K, F::Index>, (), S>,
	entries: AllocVec<Option<std::cell::UnsafeCell<Entry<V>>>, F>,
}

struct Entry<V> { usage: NonZeroUsize, hash: u64, value: V }

impl<K, V, F: FreeSetTrusted, S: Default> Default for DedupVec<K, V, F, S> {
	fn default() -> Self {
		Self {
			mapped: <_>::default(),
			entries: <_>::default(),
		}
	}
}

impl<K: Hash + Eq, V, F: FreeSetTrusted, S: BuildHasher> DedupVec<K, V, F, S> {
	pub fn new() -> Self where S: Default { Self::default() }

	pub fn with_hasher(hasher: S) -> Self {
		Self {
			mapped: HashMap::with_hasher(hasher),
			entries: <_>::default(),
		}
	}
	
	pub fn insert<Q: maps::BorrowKey<K>>(
		&mut self, pre_key: Q, make_value: impl FnOnce(Q) -> (K, V),
	) -> (F::Index, &mut K, &mut V) {
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
		Option<(F::Index, &K, &V)>
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

	pub fn remove(&mut self, id: F::Index) -> Option<V> {
		unsafe {
			let opt = self.entries.get_mut(id)?;
			let entry = &mut *opt.as_mut()?.get();

			match NonZeroUsize::new(entry.usage.get() - 1) {
				Some(sub_usage) => {
					entry.usage = sub_usage;
					None
				},

				None => {
					self.mapped
						.raw_entry_mut()
						.from_hash(entry.hash.as_(), |by| by.value == id)
						.occupied()
						.unsafe_unwrap()
						.remove();

					Some(opt.take().unsafe_unwrap().into_inner().value)
				},
			}
		}
	}

	pub unsafe fn get_unchecked(&self, id: F::Index) -> &V {
		&(&*self.entries.get_unchecked(id).unsafe_unwrap_ref().get()).value
	}

	pub unsafe fn get_unchecked_mut(&mut self, id: F::Index) -> &mut V {
		&mut
			(&mut *self.entries.get_unchecked_mut(id).unsafe_unwrap_mut().get()).value
	}

	pub fn usage(&self, id: F::Index) -> usize {
		self.entries
			.get(id)
			.and_then(|o| o.as_ref())
			.map_or(0, |e| unsafe { &*e.get() }.usage.get())
	}

	pub fn key_entries(&self) -> impl Iterator<Item = (F::Index, &K, &V)> {
		self.mapped
			.keys()
			.map(move |by| unsafe {
				let id = by.value;
				(id, &by.key, self.get_unchecked(id))
			})
	}

	pub fn key_entries_mut(&mut self) ->
		impl Iterator<Item = (F::Index, &K, &mut V)>
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

	pub fn at(&self, id: F::Index) -> Option<(&K, &V)> {
		unsafe {
			let entry = &*self.entries.get(id)?.as_ref()?.get();
			Some((Self::key_fn(id, entry, &self.mapped)(), &entry.value))
		}
	}

	pub fn at_mut(&mut self, id: F::Index) -> Option<(&K, &mut V)> {
		unsafe {
			let entry = &mut *self.entries.get(id)?.as_ref()?.get();
			Some((Self::key_fn(id, entry, &self.mapped)(), &mut entry.value))
		}
	}

	pub fn entries<'a>(&'a self) ->
		impl Iterator<Item = (F::Index, &'a V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.raw().iter()
			.zip(0..)
			.filter_map(move |(e, id)| unsafe {
				let id = NumCast::from(id).unwrap();
				let entry = &*e.as_ref()?.get();
				let key = Self::key_fn(id, entry, mapped);
				Some((id, &entry.value, key))
			})
	}

	pub fn entries_mut<'a>(&'a mut self) ->
		impl Iterator<Item = (F::Index, &'a mut V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.raw_mut().iter_mut()
			.zip(0..)
			.filter_map(move |(e, id)| unsafe {
				let id = NumCast::from(id).unwrap();
				let entry = &mut *e.as_ref()?.get();
				let key = Self::key_fn(id, entry, mapped);
				Some((id, &mut entry.value, key))
			})
	}

	unsafe fn key_fn<'a>(
		id: F::Index,
		entry: &Entry<V>,
		mapped: &'a HashMap<ByKey<K, F::Index>, (), S>,
	) -> impl FnOnce() -> &'a K {
		let hash = entry.hash;

		move || &mapped
			.raw_entry()
			.from_hash(hash, |by| by.value == id)
			.unsafe_unwrap()
			.0
			.key
	}
}

impl<K, V, F: FreeSetTrusted, S> Index<F::Index> for DedupVec<K, V, F, S> {
	type Output = V;
	fn index(&self, id: F::Index) -> &V {
		&unsafe { &*self.entries[id].as_ref().unwrap().get() }.value
	}
}

impl<K, V, F: FreeSetTrusted, S> IndexMut<F::Index> for DedupVec<K, V, F, S> {
	fn index_mut(&mut self, id: F::Index) -> &mut V {
		&mut unsafe { &mut *self.entries[id].as_mut().unwrap().get() }.value
	}
}

#[test]
fn test() {
	use std::mem::size_of;
	assert_eq!(size_of::<Entry<() >>(), size_of::<Option<Entry<() >>>());
	assert_eq!(size_of::<Entry<u32>>(), size_of::<Option<Entry<u32>>>());
}


use {
	super::*,
	std::{
		hash::{Hash, BuildHasher},
		ops::{Index, IndexMut},
		collections::HashMap,
		num::NonZeroUsize,
		cell::UnsafeCell,
		mem::{forget, replace},
	},
	num_traits::{NumCast, PrimInt},
};

pub struct DedupVec<K, V, I = usize, S = maps::std::BuildHasher> {
	mapped: HashMap<ByKey<K, I>, (), S>,
	entries: CVec<Entry<V>>,
	next_free: I,
}

pub unsafe trait TrustedIndex: NumCast + PrimInt {}
unsafe impl TrustedIndex for u8 {}
unsafe impl TrustedIndex for u16 {}
unsafe impl TrustedIndex for u32 {}
unsafe impl TrustedIndex for u64 {}
unsafe impl TrustedIndex for usize {}

struct Entry<V> { hash_or_next_free: u64, occup: Option<Occup<V>> }

struct Occup<V> { usage: NonZeroUsize, value: UnsafeCell<V> }

pub fn no_free<I: TrustedIndex>() -> I { I::max_value() }

impl<K, V, I: TrustedIndex, S: Default> Default for DedupVec<K, V, I, S> {
	fn default() -> Self {
		Self {
			mapped: <_>::default(),
			next_free: no_free(),
			entries: <_>::default(),
		}
	}
}

impl<K: Hash + Eq, V, I: TrustedIndex, S: BuildHasher> DedupVec<K, V, I, S> {
	pub fn new() -> Self where S: Default { Self::default() }

	pub fn with_hasher(hasher: S) -> Self {
		Self {
			mapped: HashMap::with_hasher(hasher),
			next_free: no_free(),
			entries: <_>::default(),
		}
	}

	fn incr_usage(occup: &mut Occup<V>) {
		let wrapping = occup.usage.get().wrapping_add(1);
		occup.usage = NonZeroUsize::new(wrapping).expect("usage overflow");
	}
	
	pub fn insert(
		&mut self,
		pre_key: &impl maps::BorrowKey<K>,
		make_value: impl FnOnce() -> (K, V),
	) -> (I, &mut K, &mut V) {
		let Self { entries, next_free, .. } = self;

		let hash = self.mapped.hasher().just_hash(&pre_key);

		let (by_key, ()) = self.mapped.raw_entry_mut()
			.from_hash(hash, |k| k.key.borrow() == pre_key)
			.and_modify(|k, ()| unsafe {
				Self::incr_usage(&mut *entries
					.get_unchecked_mut(k.value.to_usize().unsafe_unwrap())
					.occup.unsafe_unwrap_mut()
				)
			})
			.or_insert_with(|| {
				let (key, value) = make_value();

				let occup = Some(Occup {
					usage: unsafe { NonZeroUsize::new_unchecked(1) },
					value: value.into(),
				});

				let id;
				
				if *next_free == no_free() {
					id = <I as NumCast>::from(entries.len()).expect("index overflow");
					entries.push(Entry { hash_or_next_free: hash, occup });
				} else {
					id = *next_free;

					let cell = unsafe {
						entries.get_unchecked_mut(next_free.to_usize().unsafe_unwrap())
					};

					*next_free =
						NumCast::from(replace(&mut cell.hash_or_next_free, hash)).unwrap();

					forget(replace(&mut cell.occup, occup));
				};

				(ByKey::new(key, id), ())
			});

		let value = unsafe {
			&mut *entries
				.get_unchecked_mut(by_key.value.to_usize().unsafe_unwrap())
				.occup.unsafe_unwrap_mut()
				.value.get()
		};

		(by_key.value, &mut by_key.key, value)
	}

	pub fn duplicate(&mut self, id: I) -> Option<&mut V> {
		let occup = self.entries.get_mut(id.to_usize()?)?.occup.as_mut()?;
		Self::incr_usage(occup);
		Some(unsafe { &mut *occup.value.get() })
	}

	pub fn find<Q: maps::BorrowKey<K>>(&self, pre_key: &Q) ->
		Option<(I, &K, &V)>
	{
		let hash = self.mapped.hasher().just_hash(pre_key);

		self.mapped
			.raw_entry()
			.from_hash(hash, |by| by.key.borrow() == pre_key)
			.map(|(ByKey { key, value: id }, _)| unsafe {
				let value = &*self.entries
					.get_unchecked(id.to_usize().unsafe_unwrap())
					.occup.unsafe_unwrap_ref()
					.value.get();

				(*id, key, value)
			})
	}

	pub fn get(&self, id: I) -> Option<&V> {
		Some(unsafe {
			&*self.entries.get(id.to_usize()?)?.occup.as_ref()?.value.get()
		})
	}

	pub fn get_mut(&mut self, id: I) -> Option<&mut V> {
		Some(unsafe {
			&mut *self.entries.get_mut(id.to_usize()?)?.occup.as_mut()?.value.get()
		})
	}

	fn decr_usage(occup: &mut Occup<V>) -> bool {
		NonZeroUsize::new(occup.usage.get() - 1).map(|v| occup.usage = v).is_none()
	}

	unsafe fn remove_mapped(
		mapped: &mut HashMap<ByKey<K, I>, (), S>,
		entry: &mut Entry<V>,
		next_free: &mut I,
		id: I,
	) -> Option<V> {
		mapped
			.raw_entry_mut()
			.from_hash(entry.hash_or_next_free, |by| by.value == id)
			.occupied().unsafe_unwrap()
			.remove();

		entry.hash_or_next_free = next_free.to_u64().unwrap();
		*next_free = id;

		Some(entry.occup.take().unsafe_unwrap().value.into_inner())
	}

	pub fn remove(&mut self, id: I) -> Option<Option<V>> {
		unsafe {
			let en = self.entries.get_mut(id.to_usize()?)?;

			if Self::decr_usage(en.occup.as_mut()?) {
				Some(Self::remove_mapped(&mut self.mapped, en, &mut self.next_free, id))
			} else {
				Some(None)
			}
		}
	}

	pub unsafe fn remove_unchecked(&mut self, id: I) -> Option<V> {
		let en = self.entries.get_unchecked_mut(id.to_usize().unsafe_unwrap());

		if Self::decr_usage(en.occup.unsafe_unwrap_mut()) {
			Self::remove_mapped(&mut self.mapped, en, &mut self.next_free, id)
		} else {
			None
		}
	}

	pub unsafe fn get_unchecked(&self, id: I) -> &V {
		&*self.entries
			.get_unchecked(id.to_usize().unsafe_unwrap())
			.occup.unsafe_unwrap_ref()
			.value.get()
	}

	pub unsafe fn get_unchecked_mut(&mut self, id: I) -> &mut V {
		&mut *self.entries
			.get_unchecked_mut(id.to_usize().unsafe_unwrap())
			.occup.unsafe_unwrap_mut()
			.value.get()
	}

	pub fn usage(&self, id: I) -> usize {
		(|| Some(self.entries.get(id.to_usize()?)?.occup.as_ref()?.usage.get()))()
			.unwrap_or(0)
	}

	pub fn key_entries(&self) -> impl Iterator<Item = (I, &K, &V)> {
		self.mapped
			.keys()
			.map(move |by| unsafe {
				let id = by.value;
				(id, &by.key, self.get_unchecked(id))
			})
	}

	pub fn key_entries_mut(&mut self) ->
		impl Iterator<Item = (I, &K, &mut V)>
	{
		let entries = &mut self.entries;

		self.mapped
			.keys()
			.map(move |by| unsafe {
				let id = by.value;
				let value = &mut *entries
					.get_unchecked(id.to_usize().unsafe_unwrap())
					.occup.unsafe_unwrap_ref()
					.value.get();

				(id, &by.key, value)
			})
	}

	pub fn at(&self, id: I) -> Option<(&K, &V)> {
		unsafe {
			let entry = self.entries.get(id.to_usize()?)?;
			let value = &*entry.occup.as_ref()?.value.get();
			Some((Self::key_fn(id, entry, &self.mapped)(), value))
		}
	}

	pub fn at_mut(&mut self, id: I) -> Option<(&K, &mut V)> {
		unsafe {
			let entry = self.entries.get_mut(id.to_usize()?)?;
			let value = &mut *entry.occup.as_mut()?.value.get();
			Some((Self::key_fn(id, entry, &self.mapped)(), value))
		}
	}

	pub fn entries<'a>(&'a self) ->
		impl Iterator<Item = (I, &'a V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.iter()
			.zip(0..)
			.filter_map(move |(entry, id)| unsafe {
				let id = NumCast::from(id).unwrap();
				let value = &*entry.occup.as_ref()?.value.get();
				let key = Self::key_fn(id, entry, mapped);
				Some((id, value, key))
			})
	}

	pub fn entries_mut<'a>(&'a mut self) ->
		impl Iterator<Item = (I, &'a mut V, impl FnOnce() -> &'a K)>
	{
		let mapped = &self.mapped;

		self.entries
			.iter_mut()
			.zip(0..)
			.filter_map(move |(entry, id)| unsafe {
				let id = NumCast::from(id).unwrap();
				let value = &mut *entry.occup.as_ref()?.value.get();
				let key = Self::key_fn(id, entry, mapped);
				Some((id, value, key))
			})
	}

	unsafe fn key_fn<'a>(
		id: I,
		entry: &Entry<V>,
		mapped: &'a HashMap<ByKey<K, I>, (), S>,
	) -> impl FnOnce() -> &'a K {
		let hash = entry.hash_or_next_free;

		move || &mapped
			.raw_entry()
			.from_hash(hash, |by| by.value == id)
			.unsafe_unwrap()
			.0
			.key
	}
}

impl<K, V, I: TrustedIndex, S> Index<I> for DedupVec<K, V, I, S> {
	type Output = V;
	fn index(&self, id: I) -> &V {
		let id = id.to_usize().unwrap();
		unsafe { &*self.entries[id].occup.as_ref().unwrap().value.get() }
	}
}

impl<K, V, I: TrustedIndex, S> IndexMut<I> for DedupVec<K, V, I, S> {
	fn index_mut(&mut self, id: I) -> &mut V {
		let id = id.to_usize().unwrap();
		unsafe { &mut *self.entries[id].occup.as_mut().unwrap().value.get() }
	}
}

#[test]
fn test() {
	use std::mem::size_of;
	assert_eq!(size_of::<Entry<() >>(), size_of::<Option<Entry<() >>>());
	assert_eq!(size_of::<Entry<u32>>(), size_of::<Option<Entry<u32>>>());
}


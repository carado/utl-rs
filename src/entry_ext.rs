use std::hash::{Hash, BuildHasher};

pub trait EntryExt: Sized {
	type Value;
	type Key;

	type VacantEntry;
	type OccupiedEntry;

	fn vacant(self) -> Result<Self::VacantEntry, Self::OccupiedEntry>;
	fn occupied(self) -> Result<Self::OccupiedEntry, Self::VacantEntry>;

	fn vacant_ref(&self) -> Result<&Self::VacantEntry, &Self::OccupiedEntry>;
	fn occupied_ref(&self) -> Result<&Self::OccupiedEntry, &Self::VacantEntry>;

	fn vacant_mut(&mut self) ->
		Result<&mut Self::VacantEntry, &mut Self::OccupiedEntry>;
	fn occupied_mut(&mut self) ->
		Result<&mut Self::OccupiedEntry, &mut Self::VacantEntry>;

	fn occupied_value(occupied: &Self::OccupiedEntry) -> &Self::Value;
	fn occupied_value_mut(occupied: &mut Self::OccupiedEntry) -> &mut Self::Value;
	fn occupied_remove(occupied: Self::OccupiedEntry) -> (Self::Key, Self::Value);
}

pub trait EntryExtOr<Insert>: EntryExt {
	fn vacant_insert(vacant: Self::VacantEntry, value: Insert);
	fn insert_value(insert: &Insert) -> &Self::Value;
	fn insert_value_mut(insert: &mut Insert) -> &mut Self::Value;

	fn vacant_insert_into(vacant: Self::VacantEntry, insert: Insert) ->
		(Self::Key, Self::Value);

	#[inline]
	fn or(self, make_insert: impl FnOnce() -> Insert) -> EntryOr<Insert, Self> {
		match self.occupied() {
			Ok(occupied) => EntryOr::Occupied(occupied),
			Err(vacant) => EntryOr::Vacant(vacant, make_insert()),
		}
	}
}

pub enum EntryOr<I, E: EntryExtOr<I>> {
	Occupied(E::OccupiedEntry),
	Vacant(E::VacantEntry, I),
}

impl<I, E: EntryExtOr<I>> std::ops::Deref for EntryOr<I, E> {
	type Target = E::Value;
	#[inline]
	fn deref(&self) -> &E::Value {
		match self {
			Self::Occupied(occupied) => E::occupied_value(occupied),
			Self::Vacant(_, insert) => E::insert_value(insert),
		}
	}
}

impl<I, E: EntryExtOr<I>> std::ops::DerefMut for EntryOr<I, E> {
	#[inline]
	fn deref_mut(&mut self) -> &mut E::Value {
		match self {
			Self::Occupied(occupied) => E::occupied_value_mut(occupied),
			Self::Vacant(_, insert) => E::insert_value_mut(insert),
		}
	}
}

impl<I, E: EntryExtOr<I>> EntryOr<I, E> {
	#[inline]
	pub fn occupy(self) {
		if let Self::Vacant(vacant, insert) = self {
			E::vacant_insert(vacant, insert);
		}
	}

	#[inline]
	pub fn vacate_entry(self) -> (E::Key, E::Value) {
		match self {
			Self::Occupied(occupied) => E::occupied_remove(occupied),
			Self::Vacant(vacant, insert) => E::vacant_insert_into(vacant, insert),
		}
	}

	#[inline]
	pub fn vacate(self) -> E::Value { self.vacate_entry().1 }

	#[inline]
	pub fn keep_if(self, whether: bool) -> Option<(E::Key, E::Value)> {
		match whether {
			true => { self.occupy(); None },
			false => Some(self.vacate_entry()),
		}
	}
}

macro_rules! impl_pair{
	($vacant:ident, $occupied:ident($($syntax:tt)*)) => {
		#[inline]
		fn $vacant($($syntax)* self) ->
			Result<$($syntax)* Self::VacantEntry, $($syntax)* Self::OccupiedEntry>
		{
			match self {
				Self::Vacant(entry) => Ok(entry),
				Self::Occupied(entry) => Err(entry),
			}
		}

		#[inline]
		fn $occupied($($syntax)* self) ->
			Result<$($syntax)* Self::OccupiedEntry, $($syntax)* Self::VacantEntry>
		{
			match self {
				Self::Occupied(entry) => Ok(entry),
				Self::Vacant(entry) => Err(entry),
			}
		}
	};
}

macro_rules! impl_all_pairs{
	() => {
		impl_pair!{vacant    , occupied    (    )}
		impl_pair!{vacant_ref, occupied_ref(&   )}
		impl_pair!{vacant_mut, occupied_mut(&mut)}

		#[inline]
		fn occupied_value(entry: &Self::OccupiedEntry) -> &Self::Value {
			entry.get()
		}

		#[inline]
		fn occupied_value_mut(entry: &mut Self::OccupiedEntry) -> &mut Self::Value {
			entry.get_mut()
		}

		#[inline]
		fn occupied_remove(entry: Self::OccupiedEntry) -> (Self::Key, Self::Value) {
			entry.remove_entry()
		}
	}
}

mod hash_map {
	use {super::*, std::collections::hash_map::*};

	impl<'a, K: 'a, V: 'a> EntryExt for Entry<'a, K, V> {
		type Key = K;
		type Value = V;
		type VacantEntry = VacantEntry<'a, K, V>;
		type OccupiedEntry = OccupiedEntry<'a, K, V>;

		impl_all_pairs!{}
	}

	impl<'a, K: 'a, V: 'a> EntryExtOr<V> for Entry<'a, K, V> {
		#[inline]
		fn insert_value(v: &V) -> &Self::Value { v }

		#[inline]
		fn insert_value_mut(v: &mut V) -> &mut Self::Value { v }

		#[inline]
		fn vacant_insert(e: Self::VacantEntry, i: V) { e.insert(i); }

		#[inline]
		fn vacant_insert_into(vacant: Self::VacantEntry, insert: V) ->
			(Self::Key, Self::Value)
		{
			(vacant.into_key(), insert)
		}
	}

	impl<'a, K: 'a, V: 'a, S: 'a> EntryExt for RawEntryMut<'a, K, V, S> {
		type Key = K;
		type Value = V;
		type VacantEntry = RawVacantEntryMut<'a, K, V, S>;
		type OccupiedEntry = RawOccupiedEntryMut<'a, K, V, S>;

		impl_all_pairs!{}
	}

	impl<'a, K, V, S> EntryExtOr<(K, V)> for RawEntryMut<'a, K, V, S> where
		K: 'a + Hash,
		V: 'a,
		S: 'a + BuildHasher,
	{
		#[inline]
		fn insert_value((_, v): &(K, V)) -> &Self::Value { v }

		#[inline]
		fn insert_value_mut((_, v): &mut (K, V)) -> &mut Self::Value { v }

		#[inline]
		fn vacant_insert(e: Self::VacantEntry, (k, v): (K, V)) { e.insert(k, v); }

		#[inline]
		fn vacant_insert_into(_vacant: Self::VacantEntry, insert: (K, V)) ->
			(Self::Key, Self::Value)
		{
			insert
		}
	}

	impl<'a, K, V, S> EntryExtOr<(u64, K, V)> for RawEntryMut<'a, K, V, S> where
		K: 'a + Hash,
		V: 'a,
		S: 'a + BuildHasher,
	{
		#[inline]
		fn insert_value((_, _, v): &(u64, K, V)) -> &Self::Value { v }

		#[inline]
		fn insert_value_mut((_, _, v): &mut (u64, K, V)) -> &mut Self::Value { v }

		#[inline]
		fn vacant_insert(e: Self::VacantEntry, (hash, k, v): (u64, K, V)) {
			e.insert_hashed_nocheck(hash, k, v);
		}

		#[inline]
		fn vacant_insert_into(_vacant: Self::VacantEntry, (_, k, v): (u64, K, V)) ->
			(Self::Key, Self::Value)
		{
			(k, v)
		}
	}
}

mod btree_map {
	use {super::*, std::collections::btree_map::*};

	impl<'a, K: 'a + Ord, V: 'a> EntryExt for Entry<'a, K, V> {
		type Key = K;
		type Value = V;
		type VacantEntry = VacantEntry<'a, K, V>;
		type OccupiedEntry = OccupiedEntry<'a, K, V>;

		impl_all_pairs!{}
	}

	impl<'a, K: 'a + Ord, V: 'a> EntryExtOr<V> for Entry<'a, K, V> {
		#[inline]
		fn insert_value(v: &V) -> &Self::Value { v }

		#[inline]
		fn insert_value_mut(v: &mut V) -> &mut Self::Value { v }

		#[inline]
		fn vacant_insert(e: Self::VacantEntry, i: V) { e.insert(i); }

		#[inline]
		fn vacant_insert_into(vacant: Self::VacantEntry, insert: V) ->
			(Self::Key, Self::Value)
		{
			(vacant.into_key(), insert)
		}
	}
}


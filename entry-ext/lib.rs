#![feature(
	hash_raw_entry,
)]

pub trait EntryExt {
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
}

macro_rules! impl_pair{
	($vacant:ident, $occupied:ident($($syntax:tt)*)) => {
		fn $vacant($($syntax)* self) ->
			Result<$($syntax)* Self::VacantEntry, $($syntax)* Self::OccupiedEntry>
		{
			match self {
				Self::Vacant(entry) => Ok(entry),
				Self::Occupied(entry) => Err(entry),
			}
		}

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
	};
}

mod hash_map {
	use {super::*, std::collections::hash_map::*};

	impl<'a, K: 'a, V: 'a> EntryExt for Entry<'a, K, V> {
		type VacantEntry = VacantEntry<'a, K, V>;
		type OccupiedEntry = OccupiedEntry<'a, K, V>;

		impl_all_pairs!{}
	}

	impl<'a, K: 'a, V: 'a, S: 'a> EntryExt for RawEntryMut<'a, K, V, S> {
		type VacantEntry = RawVacantEntryMut<'a, K, V, S>;
		type OccupiedEntry = RawOccupiedEntryMut<'a, K, V>;

		impl_all_pairs!{}
	}
}

mod btree_map {
	use {super::*, std::collections::btree_map::*};

	impl<'a, K: 'a, V: 'a> EntryExt for Entry<'a, K, V> {
		type VacantEntry = VacantEntry<'a, K, V>;
		type OccupiedEntry = OccupiedEntry<'a, K, V>;

		impl_all_pairs!{}
	}
}

/*
mod specs_storage {
	use {super::*, specs::storage::*};

	impl<'a, 'b: 'a, T: 'a, D: 'a> EntryExt for StorageEntry<'a, 'b, T, D> {
		type VacantEntry = VacantEntry<'a, 'b, T, D>;
		type OccupiedEntry = OccupiedEntry<'a, 'b, T, D>;

		impl_all_pairs!{}
	}
}
*/


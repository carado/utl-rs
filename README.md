### carado's utl-rs

Various (not very tested and entirely undocumented) utilities for Rust.

Can be used as their own crates, or just `use ::utl::*` to re-export useful traits, types, and modules to the global scope.

* `bits-ext` implements various operations on primitive integer number types
* `by`:
	* `ByPtr` to hash/eq/compare a type by the address (rather than vuel) to which it derefs
	* `ByKey<K, V>` to hash/eq/compare only by `K`, not by `V`
	* `WeakKey` to hash/eq/compare a `std::sync::Weak` or `std::rc::Weak` by its pointer
* `cvec`: an alternative to `std::vec::Vec` that takes only two words and is a bit faster on `push`, but always has capacity `ceil(log2(length))` (and thus may reallocate more often).
* `entry-ext`: to be able to do `.occupied()` or `.vacant()` on hashmap entries
* `just-hash`: to be able to do `hash_map.hasher().just_hash(&value)`
* `maps`: a set of useful presets:
	* `sht` ("short") is based on the `fnv` hasher and is faster on shorter keys (but shouldn't be used on untrusted keysets, as collisions might be engineered).
	* `int` is based on `rustc_hash`'s `FxHasher` and is very fast on single numbers
	* `std` is just the standard hashmap using `std::collections::hash_map::RandomState` to randomize hashes per hashmap instance
	* `det` ("deterministic") is the same as `std` except the default hasher builder is always used instead of a random one
* `on-drop` lets one easily return a type that derefs, but runs a closure on its contents upon dropping
* `range-ext` implements various operations on ranges
* `rev-ord` implements `PartialOrd` and `Ord` but in reverse order to its contents (useful for `BinaryHeap`)
* `sbox` works like a `Box` to a DST except that it may remove the extra layer of indirection when the content is small (and aligned) enough; relies on `Box<dyn Trait>: Trait` to avoid branching
* `thin` works like a `Box` except it is only one `usize` long (contrary to fat pointers when DSTs are used)
* `utl` re-exports all useful traits, types, and modules


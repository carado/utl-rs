use {
	std::{collections::BinaryHeap, ops, mem::replace},
	crate::rev_ord::RevOrd,
	num_traits::NumCast,
};

pub struct AllocVec<T, F = BinaryHeap<RevOrd<usize>>> {
	data: Vec<T>,
	free: F,
}

pub trait Index = 'static + Copy + Ord + num_traits::NumCast;

pub trait FreeSet: Default {
	type Index: Index;
	fn push(&mut self, value: Self::Index);
	fn pop(&mut self) -> Option<Self::Index>;
	fn with_sorted_clear(&mut self, f: impl FnOnce(&[Self::Index]));
	fn len(&self) -> usize;
}

pub unsafe trait FreeSetTrusted: FreeSet {}

impl<T, F: FreeSet> AllocVec<T, F> where F::Index: Index {
	pub fn new() -> Self { <_>::default() }

	pub fn alloc(&mut self, value: T) -> F::Index {
		match self.free.pop() {
			Some(id) => {
				self.data[id.to_usize().unwrap()] = value;
				id
			},
			None => {
				let id = self.data.len();
				self.data.push(value);
				NumCast::from(id).unwrap()
			},
		}
	}

	pub fn is_empty(&self) -> bool { self.data.len() == self.free.len() }

	pub fn free(&mut self, index: F::Index) { self.free.push(index); }

	pub fn raw(&self) -> &[T] { &self.data }

	pub fn raw_mut(&mut self) -> &mut [T] { &mut self.data }

	pub fn raw_len(&self) -> F::Index { NumCast::from(self.data.len()).unwrap() }

	pub unsafe fn get_unchecked(&self, index: F::Index) -> &T {
		self.data.get_unchecked(index.to_usize().unwrap())
	}

	pub unsafe fn get_unchecked_mut(&mut self, index: F::Index) -> &mut T {
		self.data.get_unchecked_mut(index.to_usize().unwrap())
	}

	pub fn count(&self) -> usize { self.data.len() - self.free.len() }

	pub fn get(&self, index: F::Index) -> Option<&T> {
		self.data.get(index.to_usize().unwrap())
	}

	pub fn get_mut(&mut self, index: F::Index) -> Option<&mut T> {
		self.data.get_mut(index.to_usize().unwrap())
	}

	pub fn defragment(
		&mut self,
		mut relocate: impl FnMut(T, F::Index, F::Index) -> T,
	) -> &mut [T] {
		let data = &mut self.data;

		self.free.with_sorted_clear(|free| {
			let shrink_to = data.len() - free.len();

			let mut last_free = free.len().wrapping_sub(1);

			'relocate: for dst in free.iter().copied() {
				let dst = dst.to_usize().unwrap();

				if dst + 1 >= data.len() { break 'relocate; }

				while data.len() - 1 == free[last_free].to_usize().unwrap() {
					if dst + 1 >= data.len() { break 'relocate; }
					last_free -= 1;
					data.pop().unwrap();
				}

				let value = data.pop().unwrap();

				data[dst] = relocate(
					value,
					NumCast::from(data.len()).unwrap(),
					NumCast::from(dst).unwrap(),
				);
			}

			data.truncate(shrink_to);
		});

		data
	}
}

unsafe impl FreeSetTrusted for Vec<u32> {}
unsafe impl FreeSetTrusted for Vec<u64> {}
unsafe impl FreeSetTrusted for Vec<usize> {}

unsafe impl FreeSetTrusted for super::CVec<u32> {}
unsafe impl FreeSetTrusted for super::CVec<u64> {}
unsafe impl FreeSetTrusted for super::CVec<usize> {}

unsafe impl FreeSetTrusted for BinaryHeap<RevOrd<u32>> {}
unsafe impl FreeSetTrusted for BinaryHeap<RevOrd<u64>> {}
unsafe impl FreeSetTrusted for BinaryHeap<RevOrd<usize>> {}

impl<I: Index> FreeSet for Vec<I> {
	type Index = I;
	fn push(&mut self, value: I) { Vec::push(self, value); }
	fn pop(&mut self) -> Option<I> { Vec::pop(self) }
	fn len(&self) -> usize { Vec::len(self) }
	fn with_sorted_clear(&mut self, f: impl FnOnce(&[I])) {
		self.sort_unstable();
		f(&self);
		self.clear();
	}
}

impl<I: Index> FreeSet for super::CVec<I> {
	type Index = I;
	fn push(&mut self, value: I) { super::CVec::push(self, value); }
	fn pop(&mut self) -> Option<I> { super::CVec::pop(self) }
	fn len(&self) -> usize { (**self).len() }
	fn with_sorted_clear(&mut self, f: impl FnOnce(&[I])) {
		self.sort_unstable();
		f(&self);
		*self = super::CVec::new();
	}
}

impl<I: Index> FreeSet for BinaryHeap<RevOrd<I>> {
	type Index = I;
	fn push(&mut self, value: I) { BinaryHeap::push(self, RevOrd(value)); }
	fn pop(&mut self) -> Option<I> { BinaryHeap::pop(self).map(|RevOrd(i)| i) }
	fn len(&self) -> usize { BinaryHeap::len(self) }
	fn with_sorted_clear(&mut self, f: impl FnOnce(&[I])) {
		let mut v = replace(self, BinaryHeap::new()).into_vec();
		v.sort_by(|RevOrd(a), RevOrd(b)| a.cmp(b));
		f(unsafe { std::mem::transmute(&*v) });
		v.clear();
		*self = BinaryHeap::from(v);
	}
}

impl<T, F: FreeSet> ops::Index<F::Index> for AllocVec<T, F> where
	F::Index: Index,
{
	type Output = T;
	fn index(&self, index: F::Index) -> &T {
		&self.data[index.to_usize().unwrap()]
	}
}

impl<T, F: FreeSet> ops::IndexMut<F::Index> for AllocVec<T, F> where
	F::Index: Index,
{
	fn index_mut(&mut self, i: F::Index) -> &mut T {
		&mut self.data[i.to_usize().unwrap()]
	}
}

impl<T, F: FreeSet> Default for AllocVec<T, F> where
	F::Index: Index,
{
	fn default() -> Self { Self { data: Vec::new(), free: <_>::default() } }
}


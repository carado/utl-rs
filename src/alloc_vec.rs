use {
	std::{collections::BinaryHeap, ops, mem::replace},
	num_traits::AsPrimitive as As,
	crate::rev_ord::RevOrd,
};

pub struct AllocVec<T, F = BinaryHeap<RevOrd<usize>>> {
	data: Vec<T>,
	free: F,
}

pub trait Index = 'static + Copy + Ord + As<usize> where usize: As<Self>;

pub trait FreeSet: Default where usize: As<Self::Index> {
	type Index: Index;
	fn push(&mut self, value: Self::Index);
	fn pop(&mut self) -> Option<Self::Index>;
	fn with_sorted_clear(&mut self, f: impl FnOnce(&[Self::Index]));
	fn len(&self) -> usize;
}

impl<T, F: FreeSet> AllocVec<T, F> where F::Index: Index {
	pub fn new() -> Self { <_>::default() }

	pub fn alloc(&mut self, value: T) -> F::Index {
		match self.free.pop() {
			Some(id) => {
				self.data[id.as_()] = value;
				id
			},
			None => {
				let id = self.data.len();
				self.data.push(value);
				id.as_()
			},
		}
	}

	pub fn is_empty(&self) -> bool { self.data.len() == self.free.len() }

	pub fn free(&mut self, index: F::Index) { self.free.push(index); }

	pub fn raw(&self) -> &[T] { &self.data }

	pub fn raw_mut(&mut self) -> &mut [T] { &mut self.data }

	pub fn raw_len(&self) -> F::Index { self.data.len().as_() }

	pub unsafe fn get_unchecked(&self, index: F::Index) -> &T {
		self.data.get_unchecked(index.as_())
	}

	pub unsafe fn get_unchecked_mut(&mut self, index: F::Index) -> &mut T {
		self.data.get_unchecked_mut(index.as_())
	}

	pub fn count(&self) -> usize { self.data.len() - self.free.len() }

	pub fn get(&self, index: F::Index) -> Option<&T> {
		self.data.get(index.as_())
	}

	pub fn get_mut(&mut self, index: F::Index) -> Option<&mut T> {
		self.data.get_mut(index.as_())
	}

	pub fn defragment(
		&mut self,
		mut relocate: impl FnMut(T, F::Index, F::Index) -> T,
	) -> &mut [T] {
		let data = &mut self.data;

		self.free.with_sorted_clear(|free| {
			let shrink_to = data.len() - free.len();

			let mut last_free = free.len().wrapping_sub(1);

			'relocate: for dst in free.iter().copied().map(As::as_) {
				if dst + 1 >= data.len() { break 'relocate; }

				while data.len() - 1 == free[last_free].as_() {
					if dst + 1 >= data.len() { break 'relocate; }
					last_free -= 1;
					data.pop().unwrap();
				}

				let value = data.pop().unwrap();

				data[dst] = relocate(value, data.len().as_(), dst.as_());
			}

			data.truncate(shrink_to);
		});

		data
	}
}

impl<I> FreeSet for Vec<I> where
	I: As<usize> + Ord + Copy,
	usize: As<I>,
{
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

impl<I> FreeSet for BinaryHeap<RevOrd<I>> where
	I: As<usize> + Ord + Copy,
	usize: As<I>,
{
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
	fn index(&self, index: F::Index) -> &T { &self.data[index.as_()] }
}

impl<T, F: FreeSet> ops::IndexMut<F::Index> for AllocVec<T, F> where
	F::Index: Index,
{
	fn index_mut(&mut self, i: F::Index) -> &mut T { &mut self.data[i.as_()] }
}

impl<T, F: FreeSet> Default for AllocVec<T, F> where
	F::Index: Index,
{
	fn default() -> Self { Self { data: Vec::new(), free: <_>::default() } }
}


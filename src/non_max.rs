#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct NonMax<I = usize>(I);

impl<I: num_traits::Bounded + Eq + Copy> NonMax<I> {
	pub fn none() -> Self { NonMax(I::max_value()) }

	pub fn some(n: I) -> Self {
		assert!(n != I::max_value());
		NonMax(n)
	}

	pub fn take(&mut self) -> Option<I> {
		std::mem::replace(self, Self::none()).into()
	}

	pub fn replace(&mut self, v: I) -> Option<I> {
		std::mem::replace(self, Self::some(v)).into()
	}

	pub fn is_some(&self) -> bool { self.0 != I::max_value() }

	pub fn is_none(&self) -> bool { self.0 == I::max_value() }

	pub fn get(&self) -> Option<I> { (*self).into() }

	pub fn raw(&self) -> I { self.0 }

	pub fn unwrap(self) -> I {
		debug_assert!(self.is_some());
		self.0
	}
}

impl<I: num_traits::Bounded + Eq + Copy> Default for NonMax<I> {
	fn default() -> Self { Self::none() }
}

impl<I: num_traits::Bounded + Eq + Copy> From<Option<I>> for NonMax<I> {
	fn from(v: Option<I>) -> Self {
		match v {
			None => Self::none(),
			Some(n) => Self::some(n),
		}
	}
}

impl<I: num_traits::Bounded + Eq + Copy> Into<Option<I>> for NonMax<I> {
	fn into(self) -> Option<I> {
		if self.is_none() { None } else { Some(self.0) }
	}
}

impl<I> std::fmt::Debug for NonMax<I> where
	I: num_traits::Bounded + Eq + std::fmt::Debug + Copy,
{
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.get().fmt(fmt)
	}
}


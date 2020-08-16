use std::{ops::Range, borrow::Borrow};

pub trait RangeExt<T>: Borrow<Range<T>> {
	fn intersect(&self, b: &Self) -> bool where T: Ord;
}

impl<T> RangeExt<T> for Range<T> {
	fn intersect(&self, b: &Self) -> bool where T: Ord {
		!(self.start >= b.end || b.start >= self.end)
	}
}


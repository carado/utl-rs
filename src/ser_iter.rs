use serde::{*, ser::SerializeSeq};

pub struct SerIter<I>(std::cell::Cell<Option<I>>);

impl<I> SerIter<I> {
	pub fn new(iter: I) -> Self { Self(Some(iter).into()) }
	pub fn into_inner(self) -> Option<I> { self.0.into_inner() }
}

impl<I, T> Serialize for SerIter<I> where
	I: Iterator<Item = T>,
	T: Serialize,
{
	fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
		let mut seq = ser.serialize_seq(None)?;
		if let Some(iter) = self.0.take() {
			for element in iter {
				seq.serialize_element(&element)?;
			}
		}
		seq.end()
	}
}

pub struct SerTrustedLen<I>(std::cell::Cell<Option<I>>);

impl<I> SerTrustedLen<I> {
	pub fn new(iter: I) -> Self { Self(Some(iter).into()) }
	pub fn into_inner(self) -> Option<I> { self.0.into_inner() }
}

impl<I, T> Serialize for SerTrustedLen<I> where
	I: Iterator<Item = T>,
	T: Serialize,
{
	fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
		let mut seq;
		if let Some(iter) = self.0.take() {
			seq = ser.serialize_seq(Some(iter.size_hint().1.unwrap()))?;
			for element in iter {
				seq.serialize_element(&element)?;
			}
		} else {
			seq = ser.serialize_seq(Some(0))?;
		}
		seq.end()
	}
}


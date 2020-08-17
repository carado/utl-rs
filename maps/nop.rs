use std::mem::transmute as t;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hasher(u64);

pub type BuildHasher = std::hash::BuildHasherDefault<Hasher>;
pub type Map<K, V> = std::collections::HashMap<K, V, BuildHasher>;
pub type Set<T> = std::collections::HashSet<T, BuildHasher>;

pub fn hash(val: &(impl ?Sized + super::Hash)) -> u64 {
	super::hash::<Hasher, _>(val)
}

impl std::hash::Hasher for Hasher {
	fn finish(&self) -> u64 { self.0 }

	fn write(&mut self, bytes: &[u8]) {
		for byte in bytes.iter().cloned() {
			self.write_u8(byte);
		}
	}

	fn write_u8  (&mut self, i: u8  ) { self.0 <<=   8; self.0 |= i as u64; }
	fn write_u16 (&mut self, i: u16 ) { self.0 <<=  16; self.0 |= i as u64; }
	fn write_u32 (&mut self, i: u32 ) { self.0 <<=  32; self.0 |= i as u64; }

	fn write_u64 (&mut self, i: u64 ) { self.0 = i as u64; }
	fn write_u128(&mut self, i: u128) { self.0 = i as u64; }

	fn write_i8  (&mut self, i: i8  ) { self.write_u8  (unsafe { t(i) }); }
	fn write_i16 (&mut self, i: i16 ) { self.write_u16 (unsafe { t(i) }); }
	fn write_i32 (&mut self, i: i32 ) { self.write_u32 (unsafe { t(i) }); }
	fn write_i64 (&mut self, i: i64 ) { self.write_u64 (unsafe { t(i) }); }
	fn write_i128(&mut self, i: i128) { self.write_u128(unsafe { t(i) }); }

	fn write_usize(&mut self, i: usize) {
		self.0 <<= std::mem::size_of::<usize>() * 8;
		self.0 |= i as u64;
	}

	fn write_isize(&mut self, i: isize) { self.write_usize(unsafe { t(i) }); }
}


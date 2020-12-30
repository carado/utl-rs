use {
	crate::vec_trait::VecTrait,
	serde::Serialize,
};

#[derive(Debug)]
#[allow(unreachable_code)]
pub struct Infallible(!);

impl std::error::Error for Infallible {}

impl serde::ser::Error for Infallible {
	fn custom<T: std::fmt::Display>(_: T) -> Self { unreachable!() }
}

impl std::fmt::Display for Infallible {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.0
	}
}

pub type Result<T = ()> = std::result::Result<T, Infallible>;

type Range = std::ops::Range<usize>;

#[derive(Debug, Clone, Default)]
pub struct BytesSer<B = crate::cvec::CVec<u8>, R = crate::cvec::CVec<Range>> {
	buffer: B,
	ranges: R,
	last_start: usize,
}

pub struct BytesSerLen<'a, B, R> {
	ser: &'a mut BytesSer<B, R>,
	opt_insert_len: usize,
	len: usize,
}

impl<'a, B, R> BytesSerLen<'a, B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	fn new(ser: &'a mut BytesSer<B, R>, opt_len: Option<usize>) -> Self {
		let opt_insert_len = match opt_len {
			Some(len) => { ser.ser_usize(len); usize::max_value() },
			None => {
				let range_i = ser.ranges.len();
				ser.ranges.extend_one(usize::max_value() .. usize::max_value());
				range_i
			},
		};
		Self { ser, opt_insert_len, len: 0, }
	}

	fn end(self) {
		if self.opt_insert_len != usize::max_value() {
			let start = self.ser.buffer.len();
			self.ser.ser_usize(self.len);
			*unsafe { self.ser.ranges.get_unchecked_mut(self.opt_insert_len) } =
				start .. self.ser.buffer.len();
			self.ser.last_start = self.ser.buffer.len();
		}
	}
}

impl<B, R> BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	pub fn len(&self) -> usize { self.buffer.len() }

	pub fn slices(&self) -> impl std::iter::TrustedLen<Item = &'_ [u8]> {
		unsafe {
			self.ranges
				.iter().map(move |rng| self.buffer.get_unchecked(rng.clone()))
				.chain(std::iter::once(self.buffer.get_unchecked(self.last_start..)))
		}
	}

	pub fn bytes(&self) -> impl '_ + Iterator<Item = u8> {
		self.slices().flat_map(|slice| slice.iter().copied())
	}

	pub fn clear(&mut self) {
		self.ranges.clear();
		self.buffer.clear();
		self.last_start = 0;
	}

	pub fn new() -> Self where B: Default, R: Default { Self::default() }

	pub fn reuse_vecs(mut buffer: B, mut ranges: R) -> Self {
		buffer.clear();
		ranges.clear();
		Self { last_start: 0, buffer, ranges }
	}

	fn ecs(&mut self, s: &[u8]) { self.buffer.extend_copy_slice(s); }

	fn e1(&mut self, b: u8) { self.buffer.extend_one(b); }

	fn ser_usize(&mut self, mut v: usize) {
		loop {
			let more = v > 0x80;
			self.e1((v as u8 & 0x7F) | ((more as u8) << 7));
			if !more { break; }
			v -= 0x80;
			v >>= 7;
		}
	}

	fn ser_u16(&mut self, v: u16) {
		match v.leading_zeros() {
			0     => self.ecs(&[0x80, (v >> 8) as u8, v as u8]),
			1..=7 => self.ecs(&[0x80 | (v >> 8) as u8, v as u8]),
			8     => self.ecs(&[0x80, v as u8 & 0x7F]),
			9..   => self.e1(v as u8),
		}
	}

	fn ser_u32(&mut self, v: u32) {
		if v < 0b1_000000 {
			self.e1(v as _);
		} else {
			let zeros = v.leading_zeros();
			let bytes_m1 = (((32 + 7 - zeros) / 8) - 1) as usize; // 0..=3
			let masked = zeros & 0b111;

			let mut xor = bytes_m1 << 6;

			if masked <= 1 {
				let shift = 2 - masked;
				self.e1(((0b1_00000 | (xor as u8 >> 3)) << shift) & 0b11_000000);
				xor = ((xor ^ 0b1_000000) >> 1) << shift;
			}

			self.ecs(
				&(v ^ ((xor as u32) << (bytes_m1 * 8))).to_be_bytes()[3 - bytes_m1..]
			);
		}
	}

	fn ser_u64(&mut self, v: u64) {
		if v < 0b1_00000 {
			self.e1(v as _);
		} else {
			let zeros = v.leading_zeros();
			let bytes_m1 = (((64 + 7 - zeros) / 8) - 1) as usize; // 0..=7
			let masked = zeros & 0b111;

			let mut xor = bytes_m1 << 5;

			if masked <= 2 {
				let shift = 3 - masked;
				self.e1(((0b1_0000 | (xor as u8 >> 4)) << shift) & 0b111_00000);
				xor = ((xor ^ 0b1_00000) >> 1) << shift;
			}

			self.ecs(
				&(v ^ ((xor as u64) << (bytes_m1 * 8))).to_be_bytes()[7 - bytes_m1..]
			);
		}
	}

	fn ser_u128(&mut self, v: u128) {
		if v < 0b1_0000 {
			self.e1(v as _);
		} else {
			let zeros = v.leading_zeros();
			let bytes_m1 = (((128 + 7 - zeros) / 8) - 1) as usize; // 0..=15
			let masked = zeros & 0b111;

			let mut xor = bytes_m1 << 4;

			if masked <= 3 {
				let shift = 4 - masked;
				self.e1(((0b1_000 | (xor as u8 >> 5)) << shift) & 0b1111_0000);
				xor = ((xor ^ 0b1_0000) >> 1) << shift;
			}

			self.ecs(
				&(v ^ ((xor as u128) << (bytes_m1 * 8))).to_be_bytes()[15 - bytes_m1..]
			);
		}
	}
}

#[test]
fn test() {
	fn f(g: impl Fn(u128, &mut BytesSer<Vec<u8>>)) {
		let [mut t0, mut t1]: [BytesSer<Vec<u8>>; 2] = [<_>::default(), <_>::default()];
		g( 0, &mut t0);
		g(!0, &mut t1);
		assert_eq!(t0.len(), t1.len());
		for (b0, b1) in t0.bytes().zip(t1.bytes()) {
			for bit in (0..8).rev() {
				print!("{}", match ((b0 >> bit) & 1, (b1 >> bit) & 1) {
					(0, 0) => '0',
					(1, 1) => '1',
					(0, 1) => '.',
					_ => unreachable!(),
				});
			}
			print!(" ");
		}
	}

	for i in 0..16 {
		print!("{:3}: ", i);
		f(|b, v| {
			let n = (((1u16 << i) - 1) & b as u16) | (1u16 << i);
			v.ser_u16(n);
		});
		println!();
	}

	println!();

	for i in 0..32 {
		print!("{:3}: ", i);
		f(|b, v| {
			let n = (((1u32 << i) - 1) & b as u32) | (1u32 << i);
			v.ser_u32(n);
		});
		println!();
	}

	println!();

	for i in 0..64 {
		print!("{:3}: ", i);
		f(|b, v| {
			let n = (((1u64 << i) - 1) & b as u64) | (1u64 << i);
			v.ser_u64(n);
		});
		println!();
	}

	println!();

	for i in 0..128 {
		print!("{:3}: ", i);
		f(|b, v| {
			let n = (((1u128 << i) - 1) & b as u128) | (1u128 << i);
			v.ser_u128(n);
		});
		println!();
	}
}

fn unsign<T, U>(v: T) -> U where
	T: std::ops::Neg<Output = T> + num_traits::AsPrimitive<U>
		+ std::cmp::Ord + num_traits::Zero,
	U: std::ops::Shl<u32, Output = U> + std::ops::Add<Output = U>
		+ num_traits::One + Copy + 'static,
{
	if v < T::zero() { ((-v).as_() << 1) + U::one() } else { v.as_() << 1 }
}

impl<'a, B, R> serde::Serializer for &'a mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	type SerializeSeq = BytesSerLen<'a, B, R>;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = BytesSerLen<'a, B, R>;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result {
		self.buffer.extend_one(v as _);
		Ok(())
	}

	fn serialize_u8(self, v: u8) -> Result { Ok(self.buffer.extend_one(v     )) }
	fn serialize_i8(self, v: i8) -> Result { Ok(self.buffer.extend_one(v as _)) }

	fn serialize_u16(self, v: u16) -> Result { self.ser_u16(v); Ok(()) }
	fn serialize_i16(self, v: i16) -> Result { self.ser_u16(unsign(v)); Ok(()) }

	fn serialize_u32(self, v: u32) -> Result { self.ser_u32(v); Ok(()) }
	fn serialize_i32(self, v: i32) -> Result { self.ser_u32(unsign(v)); Ok(()) }

	fn serialize_u64(self, v: u64) -> Result { self.ser_u64(v); Ok(()) }
	fn serialize_i64(self, v: i64) -> Result { self.ser_u64(unsign(v)); Ok(()) }

	fn serialize_u128(self, v: u128) -> Result {
		self.ser_u128(v);
		Ok(())
	}

	fn serialize_i128(self, v: i128) -> Result {
		self.ser_u128(unsign(v));
		Ok(())
	}

	fn serialize_f32(self, v: f32) -> Result {
		self.ecs(&v.to_bits().to_le_bytes());
		Ok(())
	}

	fn serialize_f64(self, v: f64) -> Result {
		self.ecs(&v.to_bits().to_le_bytes());
		Ok(())
	}
	
	fn serialize_char(self, v: char) -> Result { self.ser_u32(v as _); Ok(()) }
	fn serialize_str(self, v: &str) -> Result {
		self.ser_usize(v.chars().count());
		for char in v.chars() { self.serialize_char(char)?; }
		Ok(())
		//TODO could be better ?
	}

	fn serialize_bytes(self, v: &[u8]) -> Result {
		self.ser_usize(v.len());
		self.ecs(v);
		Ok(())
	}

	fn serialize_none(self) -> Result { self.e1(0); Ok(()) }

	fn serialize_some<U: ?Sized + Serialize>(self, v: &U) -> Result {
		self.e1(1);
		v.serialize(self)
	}

	fn serialize_unit(self) -> Result { Ok(()) }

	fn serialize_unit_struct(self, _name: &'static str) -> Result { Ok(()) }

	fn serialize_unit_variant(
		self, _name: &'static str, variant_index: u32, _variant: &'static str,
	) -> Result {
		self.ser_u32(variant_index);
		Ok(())
	}

	fn serialize_newtype_struct<U: ?Sized + Serialize>(
		self, _name: &'static str, value: &U,
	) -> Result {
		value.serialize(self)
	}

	fn serialize_newtype_variant<U: ?Sized + Serialize>(
		self,
		_name: &'static str, variant_index: u32, _variant: &'static str, value: &U,
	) -> Result {
		self.ser_u32(variant_index);
		value.serialize(self)
	}

	fn serialize_seq(
		self, opt_len: Option<usize>,
	) -> Result<BytesSerLen<'a, B, R>> {
		Ok(BytesSerLen::new(self, opt_len))
	}

	fn serialize_tuple(self, _len: usize) -> Result<Self> { Ok(self) }

	fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self> {
		Ok(self)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self> {
		self.ser_u32(variant_index);
		Ok(self)
	}

	fn serialize_map(
		self, opt_len: Option<usize>,
	) -> Result<BytesSerLen<'a, B, R>> {
		Ok(BytesSerLen::new(self, opt_len))
	}

	fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self> {
		Ok(self)
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self> {
		self.ser_u32(variant_index);
		Ok(self)
	}
}

impl<'a, B, R> serde::ser::SerializeSeq for BytesSerLen<'a, B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_element<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		self.len += 1;
		value.serialize(&mut *self.ser)
	}

	fn end(self) -> Result { self.end(); Ok(()) }
}


impl<B, R> serde::ser::SerializeTuple for &'_ mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_element<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result { Ok(()) }
}

impl<B, R> serde::ser::SerializeTupleStruct for &'_ mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result { Ok(()) }
}

impl<B, R> serde::ser::SerializeTupleVariant for &'_ mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result { Ok(()) }
}

impl<'a, B, R> serde::ser::SerializeMap for BytesSerLen<'a, B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_key<U: ?Sized + Serialize>(&mut self, key: &U) -> Result {
		self.len += 1;
		key.serialize(&mut *self.ser)
	}

	fn serialize_value<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		value.serialize(&mut *self.ser)
	}

	fn end(self) -> Result { self.end(); Ok(()) }
}

impl<B, R> serde::ser::SerializeStruct for &'_ mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(
		&mut self, _key: &'static str, value: &U,
	) -> Result {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result { Ok(()) }
}

impl<B, R> serde::ser::SerializeStructVariant for &'_ mut BytesSer<B, R> where
	B: VecTrait<u8>,
	R: VecTrait<Range>,
{
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(
		&mut self, _key: &'static str, value: &U,
	) -> Result {
		value.serialize(&mut **self)
	}

	fn end(self) -> Result { Ok(()) }
}


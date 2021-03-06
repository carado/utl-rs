use {
	serde::{
		Deserializer,
		de::{
			Visitor,
			DeserializeSeed,
			IntoDeserializer,
		},
	},
	std::{
		io::Read,
		fmt::{self, Display},
	}
};

#[derive(Debug)]
pub enum Error {
	InvalidBool(u8),
	InvalidEnumDiscriminant(u32),
	Io(std::io::Error),
	Custom(String),
	AllocExceeded,
	Utf8(std::str::Utf8Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidBool(n) =>
				write!(f, "byte 0x{:02X} doesn't represent a bool", n),
			Self::InvalidEnumDiscriminant(n) =>
				write!(f, "invalid enum discriminant {}", n),
			Self::Io    (e) => write!(f, "{}", e),
			Self::Custom(e) => write!(f, "{}", e),
			Self::AllocExceeded => write!(f, "ran out of allocation"),
			Self::Utf8(e) => write!(f, "UTF-8 decoding error: {}", e),
		}
	}
}

impl serde::de::Error for Error {
	fn custom<T: Display>(msg: T) -> Self { Self::Custom(format!("{}", msg)) }
}

impl std::error::Error for Error {}

fn eof<T>() -> Result<T> {
	#[derive(Debug)]
	struct Eof;

	impl fmt::Display for Eof {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			write!(f, "unexpected end of file")
		}
	}

	impl std::error::Error for Eof {}

	Err(Error::Io(
		std::io::Error::new(std::io::ErrorKind::UnexpectedEof, Box::new(Eof))
	))
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub struct BytesDe<'de, R> { read: &'de mut R, alloc: usize }

pub struct BytesDeLen<'a, 'de, R> { len: usize, de: &'a mut BytesDe<'de, R> }

impl<'de, R: Read> BytesDe<'de, R> {
	pub fn with_alloc_limit(read: &'de mut R, limit: usize) -> Self {
		Self { read, alloc: limit }
	}

	pub fn new(read: &'de mut R) -> Self { Self::with_alloc_limit(read, 1 << 24) }

	pub fn end(self) -> &'de mut R { self.read }

	pub fn deserialize<T: serde::Deserialize<'de>>(&mut self) -> Result<T> {
		T::deserialize(self)
	}

	fn byte(&mut self) -> Result<u8> {
		let mut byte = [0u8];
		match self.read.read(&mut byte) {
			Ok(0) => eof(),
			Err(e) => Err(Error::Io(e)),
			_ => Ok(byte[0]),
		}
	}

	fn rex(&mut self, to: &mut [u8]) -> Result {
		if !to.is_empty() { self.read.read_exact(to).map_err(Error::Io)?; }
		Ok(())
	}

	fn consume_alloc(&mut self, n: usize) -> Result {
		if n > self.alloc {
			Err(Error::AllocExceeded)
		} else {
			self.alloc -= n;
			Ok(())
		}
	}

	fn de_u16(&mut self) -> Result<u16> {
		Ok(match self.byte()? {
			n@0..=0x7F => n as u16,
			0x80 => match self.byte()? {
				n@..=0x7F => n as u16 | 0x80,
				n@0x80..  => ((n as u16) << 8) | self.byte()? as u16,
			},
			n => ((n as u16 & 0x7F) << 8) | self.byte()? as u16,
		})
	}

	fn de_usize(&mut self) -> Result<usize> {
		let mut n: u64 = 0;
		let mut bits = 0;
		loop {
			let byte = self.byte()?;

			n |= ((byte & 0x7F) as u64) << bits;

			if byte >> 7 == 1 {
				n += 0x80 << bits;
				bits += 7;
			} else {
				break;
			}
		}

		Ok(n as usize)
	}

	fn de_usize_alloc(&mut self) -> Result<usize> {
		let len = self.de_usize()?;
		self.consume_alloc(len)?;
		Ok(len)
	}

	fn de_usize_buf(&mut self) -> Result<Vec<u8>> {
		let mut buf = vec![0u8; self.de_usize_alloc()?];
		self.rex(&mut buf)?;
		Ok(buf)
	}

	fn de_bool(&mut self) -> Result<bool> {
		Ok(match self.byte()? {
			0 => false,
			1 => true,
			n => return Err(Error::InvalidBool(n)),
		})
	}

	fn de_u32(&mut self) -> Result<u32> {
		let head = self.byte()?;
		if head >> 6 == 0 {
			Ok(head as _)
		} else if head & 0b111111 == 0 {
			let next = self.byte()?;
			let not_shift = head >> 7;
			let extra = ((head | (next >> 2)) >> (4 + not_shift)) & 0b11;
			let mut bytes = [0u8; 4];
			self.rex(&mut bytes[4 - extra as usize ..])?;
			Ok(
				u32::from_be_bytes(bytes)
				| (
					(((next & !(extra << (not_shift + 6))) | (0b1_000000 << not_shift)) as u32)
					<< (extra * 8)
				)
			)
		} else {
			let extra = head >> 6;
			let mut bytes = [0u8; 4];
			dbg!(extra);
			self.rex(&mut bytes[4 - extra as usize ..])?;
			Ok(u32::from_be_bytes(bytes) | ((head as u32 & 0b111111) << (extra * 8)))
		}
	}

	fn de_u64(&mut self) -> Result<u64> {
		let head = self.byte()?;
		if head >> 5 == 0 {
			Ok(head as _)
		} else if head & 0b11111 == 0 {
			let next = self.byte()?;
			let shift = head.leading_zeros();
			let extra = ((head | (next >> 3)) >> (4 - shift)) & 0b111;
			let mut bytes = [0u8; 8];
			self.rex(&mut bytes[8 - extra as usize ..])?;
			Ok(
				u64::from_be_bytes(bytes)
				| (
					(((next & (0x7F >> shift)) | (0x80 >> shift)) as u64) << (extra * 8)
				)
			)
		} else {
			let extra = head >> 5;
			let mut bytes = [0u8; 8];
			self.rex(&mut bytes[8 - extra as usize ..])?;
			Ok(u64::from_be_bytes(bytes) | ((head as u64 & 0b11111) << (extra * 8)))
		}
	}

	fn de_u128(&mut self) -> Result<u128> {
		let head = self.byte()?;
		if head >> 4 == 0 {
			Ok(head as _)
		} else if head & 0b1111 == 0 {
			let next = self.byte()?;
			let shift = head.leading_zeros();
			let extra = ((head | (next >> 4)) >> (3 - shift)) & 0b1111;
			let mut bytes = [0u8; 16];
			self.rex(&mut bytes[16 - extra as usize ..])?;
			Ok(
				u128::from_be_bytes(bytes)
				| (
					(((next & (0x7F >> shift)) | (0x80 >> shift)) as u128) << (extra * 8)
				)
			)
		} else {
			let extra = head >> 4;
			let mut bytes = [0u8; 16];
			self.rex(&mut bytes[16 - extra as usize ..])?;
			Ok(u128::from_be_bytes(bytes) | ((head as u128 & 0b1111) << (extra * 8)))
		}
	}
}

fn resign<T, U>(v: T) -> U where
	T: num_traits::One + std::ops::BitAnd<Output = T>
		+ std::ops::Shr<u32, Output = T> + num_traits::AsPrimitive<U> + Eq,
	U: std::ops::Not<Output = U> + Copy + 'static,
{
	if v & T::one() == T::one() {
		!(v >> 1).as_()
	} else {
		(v >> 1).as_()
	}
}

impl<'a, 'de, R: Read> Deserializer<'de> for &'a mut BytesDe<'de, R> {
	type Error = Error;

	fn deserialize_bool<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_bool(self.de_bool()?)
	}

	fn deserialize_u8<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_u8(self.byte()?     )
	}

	fn deserialize_i8<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_i8(self.byte()? as _)
	}

	fn deserialize_u16<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_u16(self.de_u16()?)
	}

	fn deserialize_i16<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_i16(resign(self.de_u16()?))
	}

	fn deserialize_u32<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_u32(self.de_u32()?)
	}

	fn deserialize_i32<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_i32(resign(self.de_u32()?))
	}

	fn deserialize_u64<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_u64(self.de_u64()?)
	}

	fn deserialize_i64<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_i64(resign(self.de_u64()?))
	}

	fn deserialize_u128<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_u128(self.de_u128()?)
	}

	fn deserialize_i128<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_i128(resign(self.de_u128()?))
	}

	fn deserialize_f32<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		let mut bytes = [0u8; 4];
		self.rex(&mut bytes)?;
		v.visit_f32(f32::from_bits(u32::from_le_bytes(bytes)))
	}

	fn deserialize_f64<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		let mut bytes = [0u8; 8];
		self.rex(&mut bytes)?;
		v.visit_f64(f64::from_bits(u64::from_le_bytes(bytes)))
	}

	fn deserialize_char<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_char(match self.byte()? {
			n@0..=0x7F => n as u8 as char,
			n => {
				let mut bytes = [0u8; 6];
				bytes[0] = n;
				let len = n.leading_ones() as usize;
				self.rex(&mut bytes[1 .. len])?;
				match std::str::from_utf8(&bytes[.. len])
					.map_err(Error::Utf8)?.chars().next()
				{
					Some(char) => char,
					None => return eof(),
				}
			},
		})
	}

	fn deserialize_str<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_str(
			std::str::from_utf8(&self.de_usize_buf()?).map_err(Error::Utf8)?
		)
	}

	fn deserialize_string<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_string(
			String::from_utf8(self.de_usize_buf()?)
				.map_err(|e| Error::Utf8(e.utf8_error()))?
		)
	}

	fn deserialize_bytes<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_bytes(&self.de_usize_buf()?)
	}

	fn deserialize_byte_buf<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_byte_buf(self.de_usize_buf()?)
	}

	fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		if self.de_bool()? {
			v.visit_some(self)
		} else {
			v.visit_none()
		}
	}

	fn deserialize_unit<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_unit()
	}

	fn deserialize_unit_struct<V: Visitor<'de>>(
		self, _name: &'static str, v: V,
	) -> Result<V::Value> {
		self.deserialize_unit(v)
	}

	fn deserialize_newtype_struct<V: Visitor<'de>>(
		self, _name: &'static str, v: V,
	) -> Result<V::Value> {
		v.visit_newtype_struct(self)
	}

	fn deserialize_seq<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		let len = self.de_usize_alloc()?;
		v.visit_seq(BytesDeLen { len, de: self })
	}

	fn deserialize_tuple<V: Visitor<'de>>(
		self, len: usize, v: V,
	) -> Result<V::Value> {
		v.visit_seq(BytesDeLen { len, de: self })
	}

	fn deserialize_tuple_struct<V: Visitor<'de>>(
		self, _name: &'static str, len: usize, v: V,
	) -> Result<V::Value> {
		v.visit_seq(BytesDeLen { len, de: self })
	}

	fn deserialize_map<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		let len = self.de_usize_alloc()?;
		v.visit_seq(BytesDeLen { len, de: self })
	}

	fn deserialize_struct<V: Visitor<'de>>(
		self, _name: &'static str, fields: &'static [&'static str], v: V,
	) -> Result<V::Value> {
		v.visit_seq(BytesDeLen { len: fields.len(), de: self })
	}

	fn deserialize_enum<V: Visitor<'de>>(
		self, _name: &'static str, _variants: &'static [&'static str], v: V,
	) -> Result<V::Value> {
		v.visit_enum(self)
	}

	fn deserialize_identifier<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
		unimplemented!("deserialize_identifier unsupported")
	}

	fn deserialize_ignored_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
		unimplemented!("deserialize_ignored_any unsupported")
	}

	fn deserialize_any<V: Visitor<'de>>(self, _: V) -> Result<V::Value> {
		unimplemented!("deserialize_any unsupported")
	}

	fn is_human_readable(&self) -> bool { false }
}

impl<'a, 'de, R> serde::de::SeqAccess<'de> for BytesDeLen<'a, 'de, R> where
	R: Read,
{
	type Error = Error;

	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>> where
		T: serde::de::DeserializeSeed<'de>,
	{
		Ok(if self.len == 0 {
			None
		} else {
			self.len -= 1;
			Some(seed.deserialize(&mut *self.de)?)
		})
	}

	fn size_hint(&self) -> Option<usize> { Some(self.len) }
}

impl<'a, 'de, R> serde::de::MapAccess<'de> for BytesDeLen<'a, 'de, R> where
	R: Read,
{
	type Error = Error;

	fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>> where
		T: serde::de::DeserializeSeed<'de>,
	{
		Ok(if self.len == 0 {
			None
		} else {
			self.len -= 1;
			Some(seed.deserialize(&mut *self.de)?)
		})
	}

	fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value> where
		T: serde::de::DeserializeSeed<'de>,
	{
		Ok(seed.deserialize(&mut *self.de)?)
	}

	fn size_hint(&self) -> Option<usize> { Some(self.len) }
}

impl<'a, 'de, R> serde::de::EnumAccess<'de> for &'a mut BytesDe<'de, R> where
	R: Read,
{
	type Error = Error;
	type Variant = Self;

	fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)> where
		V: DeserializeSeed<'de>,
	{
		let idx = self.de_u32()?;
		Ok((seed.deserialize(idx.into_deserializer())?, self))
	}
}

impl<'a, 'de, R> serde::de::VariantAccess<'de> for &'a mut BytesDe<'de, R> where
	R: Read,
{
	type Error = Error;

	fn unit_variant(self) -> Result { Ok(()) }

	fn newtype_variant_seed<V>(self, seed: V) -> Result<V::Value> where
		V: DeserializeSeed<'de>,
	{
		seed.deserialize(self)
	}

	fn tuple_variant<V>(self, len: usize, seed: V) -> Result<V::Value> where
		V: Visitor<'de>,
	{
		self.deserialize_tuple(len, seed)
	}

	fn struct_variant<V>(self, fields: &'static [&'static str], seed: V) ->
		Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_tuple(fields.len(), seed)
	}
}


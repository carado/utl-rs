use std::io::Write;
use crate::extend_ext::ExtendExt;
use serde::Serialize;

#[derive(Debug)]
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

pub struct BytesSer<T>(pub T);

impl<T: ExtendExt<u8>> BytesSer<T> {
	fn ecs(&mut self, s: &[u8]) { self.0.extend_copy_slice(s); }
	fn e1(&mut self, b: u8) { self.0.extend_one(b); }

	fn ser_u16(&mut self, v: u16) {
		match v.leading_zeros() {
			0     => self.ecs(&[0x80, (v >> 8) as u8, v as u8]),
			1..=7 => self.ecs(&[0x80 | (v >> 8) as u8, v as u8]),
			8     => self.ecs(&[0x80, v as u8 & 0x7F]),
			9..   => self.e1(v as u8),
		}
	}

	fn ser_u32(&mut self, v: u32) {
		let zeros = v.leading_zeros();
		let bytes_m1 = ((32 + 9 - zeros) / 8).saturating_sub(1) as usize; // 0..=3
		let masked = zeros & 0b111;

		let mut xor = bytes_m1 << 6;

		if masked <= 1 {
			let shift = 1 + (masked ^ 1);
			self.e1((xor >> shift) as u8 & 0b11_000000);
			xor = (xor ^ 0b1_000000) << masked;
		}

		self.ecs(
			&(v ^ ((xor as u32) << (bytes_m1 * 8))).to_be_bytes()[3 - bytes_m1..]
		);
	}

	fn ser_u64(&mut self, v: u64) {
		/*
		let [a, b, c, d, e, f, g, h] = v.to_be_bytes();
		match v.leading_zeros() {
			 0      => self.ecs(&[0xE0, a, b, c, d, e, f, g, h]),
			 1      => self.ecs(&[0x60, 0x80 | a, b, c, d, e, f, g, h]),
			 2      => self.ecs(&[0x20, 0xC0 | a, b, c, d, e, f, g, h]),
			 3..= 7 => self.ecs(&[0xE0 | a, b, c, d, e, f, g]),
			 8      => self.ecs(&[0x60, 0xC0 ^ a, b, c, d, e, f, g]),
			 9      => self.ecs(&[0x20, 0x
		}
		*/
	}
}

#[test]
fn test() {
	for i in u16::min_value()..=u16::max_value() {
		//assert_eq!(
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

impl<T: ExtendExt<u8>> serde::Serializer for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result { Ok(self.0.extend_one(v as _)) }

	fn serialize_u8(self, v: u8) -> Result { Ok(self.0.extend_one(v     )) }
	fn serialize_i8(self, v: i8) -> Result { Ok(self.0.extend_one(v as _)) }

	fn serialize_u16(self, v: u16) -> Result { self.ser_u16(v); Ok(()) }
	fn serialize_i16(self, v: i16) -> Result { self.ser_u16(unsign(v)); Ok(()) }

	fn serialize_u32(self, v: u32) -> Result { self.ser_u32(v); Ok(()) }
	fn serialize_i32(self, v: i32) -> Result { self.ser_u32(unsign(v)); Ok(()) }

	fn serialize_u64(self, v: u64) -> Result { todo!() }
	fn serialize_i64(self, v: i64) -> Result { todo!() }
	//fn serialize_u128(self, v: u128) -> Result { todo!() }
	//fn serialize_i128(self, v: i128) -> Result { todo!() }

	fn serialize_f32(self, v: f32) -> Result { todo!() }
	fn serialize_f64(self, v: f64) -> Result { todo!() }
	
	fn serialize_char(self, v: char) -> Result { todo!() }
	fn serialize_str(self, v: &str) -> Result { todo!() }
	fn serialize_bytes(self, v: &[u8]) -> Result { todo!() }
	fn serialize_none(self) -> Result { todo!() }
	fn serialize_some<U: ?Sized + Serialize>(self, v: &U) -> Result { todo!() }
	fn serialize_unit(self) -> Result { todo!() }
	fn serialize_unit_struct(self, _name: &'static str) -> Result { todo!() }

	fn serialize_unit_variant(
		self, _name: &'static str, _variant_index: u32, variant: &'static str,
	) -> Result {
		//self.serialize_str(variant)
		todo!()
	}

	fn serialize_newtype_struct<U: ?Sized + Serialize>(
		self, _name: &'static str, value: &U,
	) -> Result {
		todo!()
	}

	fn serialize_newtype_variant<U: ?Sized + Serialize>(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		value: &U,
	) -> Result {
		todo!()
	}

	fn serialize_seq(self, _len: Option<usize>) -> Result<Self> {
		todo!()
	}

	fn serialize_tuple(self, len: usize) -> Result<Self> {
		todo!()
	}

	fn serialize_tuple_struct(self, _: &'static str, len: usize) -> Result<Self> {
		todo!()
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		todo!()
		//Ok(self)
	}

	fn serialize_map(self, _len: Option<usize>) -> Result<Self> { todo!() }

	fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self> {
		todo!()
	}

	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		variant: &'static str,
		_len: usize,
	) -> Result<Self> {
		todo!()
	}
}

impl<T: ExtendExt<u8>> serde::ser::SerializeSeq for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_element<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}


impl<T: ExtendExt<u8>> serde::ser::SerializeTuple for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_element<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}

impl<T: ExtendExt<u8>> serde::ser::SerializeTupleStruct for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}

impl<T: ExtendExt<u8>> serde::ser::SerializeTupleVariant for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}

impl<T: ExtendExt<u8>> serde::ser::SerializeMap for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_key<U: ?Sized + Serialize>(&mut self, key: &U) -> Result {
		todo!()
	}

	fn serialize_value<U: ?Sized + Serialize>(&mut self, value: &U) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}

impl<T: ExtendExt<u8>> serde::ser::SerializeStruct for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(
		&mut self, key: &'static str, value: &U,
	) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}

impl<T: ExtendExt<u8>> serde::ser::SerializeStructVariant for &'_ mut BytesSer<T> {
	type Ok = ();
	type Error = Infallible;

	fn serialize_field<U: ?Sized + Serialize>(
		&mut self, key: &'static str, value: &U,
	) -> Result {
		todo!()
	}

	fn end(self) -> Result { todo!() }
}






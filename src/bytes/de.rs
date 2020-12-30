use {
	serde::{
		Deserialize,
		Deserializer,
		de::Visitor,
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
	MalformedUtf8([u8; 6]),
	Io(std::io::Error),
	Custom(String),
	SizeExceeded(u64),
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
			Self::SizeExceeded(n) => write!(f, "size header {} exceeds max", n),
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

pub struct BytesDe<'de, R> {
	read: &'de mut R,
	alloc: usize,
}

const SIZE_HEADER_MAX: u64 = 1 << 24;

impl<'de, R: Read> BytesDe<'de, R> {
	fn byte(&mut self) -> Result<u8> {
		let mut byte = [0u8];
		match self.read.read(&mut byte) {
			Ok(0) => eof(),
			Err(e) => Err(Error::Io(e)),
			_ => Ok(byte[0]),
		}
	}

	fn de_u16(&mut self) -> Result<u16> {
		Ok(match self.byte()? {
			n@0..=0x7F => n as u16,
			0x80 => match self.byte()? {
				n@..=0x7F => n as u16 | 0x80,
				n@0x80..  => ((n as u16) << 8) | self.byte()? as u16,
			},
			n => (n as u16 & 0x7F) | self.byte()? as u16,
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

		if n > SIZE_HEADER_MAX {
			Err(Error::SizeExceeded(n))
		} else {
			Ok(n as usize)
		}
	}

	fn de_u32(&mut self) -> Result<u32> {
		let head = self.byte()?;
		if head >> 6 == 0 {
			Ok(head as _)
		} else if head & 0b111111 == 0 {
			let next = self.byte()?;
			let not_shift = head >> 7;
			let extra = (head | (next >> 2)) >> (4 + not_shift);
			let mut bytes = [0u8; 4];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
			Ok(
				u32::from_be_bytes(bytes)
				| (
					(((next & !(not_shift << 7)) | (0b1_000000 << not_shift)) as u32)
					<< (extra * 8)
				)
			)
		} else {
			let extra = head >> 6;
			let mut bytes = [0u8; 4];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
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
			let extra = (head | (next >> 3)) >> (4 - shift);
			let mut bytes = [0u8; 8];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
			Ok(
				u64::from_be_bytes(bytes)
				| (
					(((next & (0x8F >> shift)) | (0x80 >> shift)) as u64) << (extra * 8)
				)
			)
		} else {
			let extra = head >> 5;
			let mut bytes = [0u8; 8];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
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
			let extra = (head | (next >> 4)) >> (3 - shift);
			let mut bytes = [0u8; 16];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
			Ok(
				u128::from_be_bytes(bytes)
				| (
					(((next & (0x8F >> shift)) | (0x80 >> shift)) as u128) << (extra * 8)
				)
			)
		} else {
			let extra = head >> 4;
			let mut bytes = [0u8; 16];
			self.read.read_exact(&mut bytes[extra as usize ..]).map_err(Error::Io)?;
			Ok(u128::from_be_bytes(bytes) | ((head as u128 & 0b1111) << (extra * 8)))
		}
	}
}

fn resign<T, U>(v: T) -> U where
	T: num_traits::One + std::ops::BitAnd<Output = T>
		+ std::ops::Shr<u32, Output = T> + num_traits::AsPrimitive<U> + Eq,
	U: std::ops::Neg<Output = U> + Copy + 'static,
{
	if v & T::one() == T::one() {
		-(v >> 1).as_()
	} else {
		(v >> 1).as_()
	}
}

impl<'de, R: Read> Deserializer<'de> for BytesDe<'de, R> {
	type Error = Error;

	fn deserialize_bool<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		match self.byte()? {
			0 => v.visit_bool(false),
			1 => v.visit_bool(true ),
			n => Err(Error::InvalidBool(n)),
		}
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

	fn deserialize_char<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		v.visit_char(match self.byte()? {
			n@0..=0x7F => n as u8 as char,
			n => {
				let mut bytes = [0u8; 6];
				bytes[0] = n;
				let range = 1 .. n.leading_ones() as usize;
				self.read.read_exact(&mut bytes[range]).map_err(Error::Io)?;
				match
					std::str::from_utf8(&bytes[range]).map_err(Error::Utf8)?.chars().next()
				{
					Some(char) => char,
					None => return eof(),
				}
			},
		})
	}

	fn deserialize_str<V: Visitor<'de>>(self, v: V) -> Result<V::Value> {
		todo!()
	}
}


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
	Low(std::io::Error),
	Custom(String),
	SizeExceeded(usize),
}

impl Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidBool(n) =>
				write!(f, "byte 0x{:02X} doesn't represent a bool", n),
			Self::InvalidEnumDiscriminant(n) =>
				write!(f, "invalid enum discriminant {}", n),
			Self::Low   (e) => write!(f, "{}", e),
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

	Err(Error::Low(
		std::io::Error::new(std::io::ErrorKind::UnexpectedEof, Box::new(Eof))
	))
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub struct BytesDe<'de, R> {
	read: &'de mut R,
	alloc: usize,
}

impl<'de, R: Read> BytesDe<'de, R> {
	fn byte(&mut self) -> Result<u8> {
		let mut byte = [0u8];
		match self.read.read(&mut byte) {
			Ok(0) => eof(),
			Err(e) => Err(Error::Io(e)),
			_ => Ok(byte[0]),
		}
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
}


use {
	serde::{
		Deserialize,
		Deserializer,
	},
	std::io::Read,
};

pub enum Error {
	InvalidBool(u8),
	InvalidEnumDiscriminant(u32),
	Io(std::io::Error),
}

fn eof() -> Error {
	#[derive(Debug)]
	struct Eof;

	impl std::fmt::Display for Eof {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "unexpected end of file")
		}
	}

	impl std::error::Error for Eof {}

	Error::Io(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, Box::new(Eof)))
}

pub type Result<T = ()> = std::result::Result<T, Error>;

pub struct BytesDe<'de, R: Read> {
	read: &'de mut R,
}




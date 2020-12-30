use {
	serde::Deserialize,
};

pub enum Error {
	Io(std::io::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;




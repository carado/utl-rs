mod ser;
mod de;

pub use self::{
	ser::BytesSer,
	de::{BytesDe, Error},
};

#[test]
fn test() {
	fn ck<T: serde::Serialize + Eq>(x: T) {
		let mut ser = BytesSer::new();
		ser.serilaize(&x);
		let mut data = ser.bytes().collect::<Vec<u8>>();
		let de = BytesDe::new(&*data);
	}
}


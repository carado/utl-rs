mod ser;
mod de;

pub use self::{
	ser::BytesSer,
	de::{BytesDe, Error},
};

#[test]
fn test() {
	fn ck<T>(x: T) where
		T: serde::Serialize + for<'de> serde::Deserialize<'de> + Eq + std::fmt::Debug,
	{
		let mut ser: BytesSer = BytesSer::new();
		ser.serialize(&x).unwrap();
		let data = ser.bytes().collect::<Vec<u8>>();
		let mut slice = &*data;
		let mut de = BytesDe::new(&mut slice);
		assert_eq!(de.deserialize::<T>().unwrap(), x);
		assert!(slice.is_empty());
	}

	for i in u8  ::min_value() ..= u8  ::max_value() { ck(i); }
	for i in i8  ::min_value() ..= i8  ::max_value() { ck(i); }
	for i in u16 ::min_value() ..= u16 ::max_value() { ck(i); }
	for i in i16 ::min_value() ..= i16 ::max_value() { ck(i); }

	//for i in u32 ::min_value() ..= u32 ::max_value() { ck(i); }
	//for i in i32 ::min_value() ..= i32 ::max_value() { ck(i); }
	//for i in u64 ::min_value() ..= u64 ::max_value() { ck(i); }
	//for i in i64 ::min_value() ..= i64 ::max_value() { ck(i); }
	//for i in u128::min_value() ..= u128::max_value() { ck(i); }
	//for i in i128::min_value() ..= i128::max_value() { ck(i); }
}


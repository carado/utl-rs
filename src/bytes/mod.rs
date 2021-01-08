mod ser;
mod de;

//TODO switch to little-endian

pub use self::{
	ser::BytesSer,
	de::{BytesDe, Error},
};

#[test]
fn test() {
	fn ck<T>(x: T) where
		T: serde::Serialize + for<'de> serde::Deserialize<'de> + Eq + std::fmt::Debug,
	{
		eprint!("{}: {:?}", std::any::type_name::<T>(), &x);
		let mut ser: BytesSer = BytesSer::new();
		ser.serialize(&x).unwrap();
		let data = ser.bytes().collect::<Vec<u8>>();
		let mut slice = &*data;
		eprint!(" →"); for byte in slice.iter() { eprint!(" {:08b}", *byte); }
		let mut de = BytesDe::new(&mut slice);
		let y = de.deserialize::<T>().unwrap();
		eprint!(" → {:?} (+{})", &y, slice.len());
		assert_eq!(x, y);
		assert!(slice.is_empty());
		eprintln!("");
	}

	fn rand_u16() -> u16 {
		let mut rng = rand::thread_rng();
		(rng.gen::<u16>() | (1 << 15)) >> rng.gen_range(0..16)
	}

	use rand::prelude::*;

	let mut rng = rand::thread_rng();

	for i in 0..128 {
		for _ in 0..16 {
			let n = (rng.gen::<u128>() | (1 << 127)) >> (127 - i);
			if n <=    u16 ::max_value() as u128           { ck(  n as u16  ); }
			if n <=    u32 ::max_value() as u128           { ck(  n as u32  ); }
			if n <=    u64 ::max_value() as u128           { ck(  n as u64  ); }
			if n <=    u128::max_value() as u128           { ck(  n as u128 ); }
			if n <  (-(i16 ::min_value() as i128)) as u128 { ck(-(n as i16 )); }
			if n <  (-(i32 ::min_value() as i128)) as u128 { ck(-(n as i32 )); }
			if n <  (-(i64 ::min_value() as i128)) as u128 { ck(-(n as i64 )); }
			if n <=    i128::max_value() as u128           { ck(-(n as i128)); }
		}
	}

	for _ in 0..1 << 12 {
		ck(rng.gen::<u8>());
		ck(rng.gen::<i8>());
		ck(((),));
		ck((rand_u16(),));
		ck((rand_u16(), rand_u16()));
		ck((rand_u16(), rand_u16(), rand_u16()));
		ck((rand_u16(), rand_u16(), rand_u16(), rand_u16()));
		ck(vec![rand_u16()]);
		ck(vec![rand_u16(), rand_u16()]);
		ck(vec![rand_u16(), rand_u16(), rand_u16()]);
		ck(vec![rand_u16(), rand_u16(), rand_u16(), rand_u16()]);
		ck(rng.gen::<char>());
		ck(Ok ::<u16, u16>(rand_u16()));
		ck(Err::<u16, u16>(rand_u16()));
		ck(None::<u16>);
		ck(Some(rand_u16()));
		ck((0..)
			.take(rng.gen_range(0..64))
			.map(|_| thread_rng().gen::<char>())
			.collect::<String>()
		);
	}
}


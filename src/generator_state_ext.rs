use std::ops::GeneratorState;

pub trait GeneratorStateExt {
	type Yielded;
	type Complete;
	fn yielded (self) -> Result<Self::Yielded , Self::Complete>;
	fn complete(self) -> Result<Self::Complete, Self::Yielded >;
}

impl<Y, R> GeneratorStateExt for GeneratorState<Y, R> {
	type Yielded  = Y;
	type Complete = R;

	fn yielded (self) -> Result<Y, R> {
		match self { Self::Yielded (v) => Ok (v), Self::Complete(v) => Err(v) }
	}

	fn complete(self) -> Result<R, Y> {
		match self { Self::Complete(v) => Ok (v), Self::Yielded (v) => Err(v) }
	}
}


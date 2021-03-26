pub trait OptionExt<T> {
	fn unwrap_none(self);
	fn expect_none(self, error: &str);
}

impl<T> OptionExt<T> for Option<T> {
	fn unwrap_none(self) {
		assert!(self.is_none());
	}

	fn expect_none(self, error: &str) {
		assert!(self.is_none(), "{}", error);
	}
}


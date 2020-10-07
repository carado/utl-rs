use {
	std::{
		marker::PhantomData,
		any::TypeId,
		cell::UnsafeCell,
		mem::MaybeUninit,
		sync::Once,
	},
	crate::*,
};

static mut AIDTIVE: MaybeUninit<parking_lot::Mutex<maps::std::Set<TypeId>>> =
	MaybeUninit::uninit();

pub struct Coffer<T, ID: 'static> {
	value: UnsafeCell<T>,
	_key: PhantomData<ID>,
}

pub struct Key<ID: 'static>(PhantomData<ID>);

impl<T: Default, ID: 'static> Default for Coffer<T, ID> {
	fn default() -> Self {
		Self { value: T::default().into(), _key: PhantomData }
	}
}

unsafe impl<T: Sync, ID: 'static> Sync for Coffer<T, ID> {}
unsafe impl<T: Send, ID: 'static> Send for Coffer<T, ID> {}

impl<T, ID: 'static> From<T> for Coffer<T, ID> {
	fn from(value: T) -> Self { Self { value: value.into(), _key: PhantomData } }
}

impl<T, ID: 'static> Coffer<T, ID> {
	pub fn new(value: T) -> Self { Self::from(value) }

	pub fn into_inner(self) -> T { self.value.into_inner() }

	pub fn get<'a>(&'a self, _: &'a mut Key<ID>) -> &'a mut T {
		unsafe { &mut *self.value.get() }
	}

	pub fn get_mut(&mut self) -> &mut T {
		unsafe { &mut *self.value.get() }
	}
}

impl<ID: 'static> Key<ID> {
	pub fn unique() -> Option<Self> {
		unsafe {
			static INIT: Once = Once::new();

			INIT.call_once(|| {
				AIDTIVE.write(<_>::default());
			});

			AIDTIVE
				.assume_init_mut()
				.lock()
				.insert(TypeId::of::<ID>())
				.then_some(Key(PhantomData))
		}
	}
}

impl<ID: 'static> Drop for Key<ID> {
	fn drop(&mut self) {
		unsafe {
			AIDTIVE
				.assume_init_mut()
				.lock()
				.remove(&TypeId::of::<ID>());
		}
	}
}


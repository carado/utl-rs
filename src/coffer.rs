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

static mut CLEFS: MaybeUninit<parking_lot::Mutex<maps::std::Set<TypeId>>> =
	MaybeUninit::uninit();

pub struct Coffer<T, C: 'static> {
	value: UnsafeCell<T>,
	_key: PhantomData<C>,
}

pub struct Clef<C: 'static>(PhantomData<C>);

impl<T: Default, C: 'static> Default for Coffer<T, C> {
	fn default() -> Self {
		Self { value: T::default().into(), _key: PhantomData }
	}
}

unsafe impl<T: Sync, C: 'static> Sync for Coffer<T, C> {}
unsafe impl<T: Send, C: 'static> Send for Coffer<T, C> {}

impl<T, C: 'static> From<T> for Coffer<T, C> {
	fn from(value: T) -> Self { Self { value: value.into(), _key: PhantomData } }
}

impl<T, C: 'static> Coffer<T, C> {
	pub fn new(value: T) -> Self { Self::from(value) }

	pub fn into_inner(self) -> T { self.value.into_inner() }

	pub fn get<'a>(&'a self, _: &'a mut Clef<C>) -> &'a mut T {
		unsafe { &mut *self.value.get() }
	}

	pub fn get_mut(&mut self) -> &mut T {
		unsafe { &mut *self.value.get() }
	}
}

impl<C: 'static> Clef<C> {
	pub fn unique() -> Option<Self> {
		unsafe {
			static INIT: Once = Once::new();

			INIT.call_once(|| {
				CLEFS.write(<_>::default());
			});

			CLEFS
				.assume_init_mut()
				.lock()
				.insert(TypeId::of::<C>())
				.then_some(Clef(PhantomData))
		}
	}
}

impl<C: 'static> Drop for Clef<C> {
	fn drop(&mut self) {
		unsafe {
			CLEFS
				.assume_init_mut()
				.lock()
				.remove(&TypeId::of::<C>());
		}
	}
}


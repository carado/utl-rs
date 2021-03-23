pub trait IsDefault {
	fn is_default(&self) -> bool;
}

macro_rules! basic_is_default {
	(
		$self:ident;
		$(
			{
				$(
					$ty:ty
					$([$($gen:tt)*])?
				),*$(,)?
			} => $expr:expr;
		)*
	) => {
		$(
			$(
				impl$(<$($gen)*>)? IsDefault for $ty {
					#[inline]
					fn is_default(&$self) -> bool { $expr }
				}
			)*
		)*
	};
}

basic_is_default!{
	self;
	{
		(),bool,
		u8,u16,u32,u64,u128,
		i8,i16,i32,i64,i128,
		f32,f64,
	} => *self == Self::default();
	{
		&'a T['a, T: IsDefault], &'a mut T['a, T: IsDefault],
		Box<T>[T: IsDefault],
		std::sync::Arc<T>[T: IsDefault], std::rc::Rc<T>[T: IsDefault],
	} => (**self).is_default();
	{ Option<T>[T] } => self.is_none();
	{
		std::collections::HashMap<K, V>[K, V],
		std::collections::BTreeMap<K, V>[K, V],
		std::collections::HashSet<T>[T],
		std::collections::BTreeSet<T>[T],
		std::collections::BinaryHeap<T>[T],
		std::collections::VecDeque<T>[T],
		std::collections::LinkedList<T>[T],
	} => self.is_empty();
	{ Result<T, U>[T: IsDefault, U] } => match self {
		Ok(v) => v.is_default(),
		Err(_) => false,
	};
}

macro_rules! tuple_is_default {
	($($($param:ident)*,)*) => {
		$(
			impl<$($param: IsDefault),*> IsDefault for ($($param,)*) {
				#[allow(non_snake_case)]
				fn is_default(&self) -> bool {
					let ($($param,)*) = self;
					$( $param.is_default() && )* true
				}
			}
		)*
	};
}

tuple_is_default!{
	A,
	A B,
	A B C,
	A B C D,
	A B C D E,
	A B C D E F,
	A B C D E F G,
	A B C D E F G H,
	A B C D E F G H I,
	A B C D E F G H I J,
	A B C D E F G H I J K,
	A B C D E F G H I J K L,
	A B C D E F G H I J K L M,
	A B C D E F G H I J K L M N,
	A B C D E F G H I J K L M N O,
	A B C D E F G H I J K L M N O P,
	A B C D E F G H I J K L M N O P Q,
	A B C D E F G H I J K L M N O P Q R,
	A B C D E F G H I J K L M N O P Q R S,
	A B C D E F G H I J K L M N O P Q R S T,
	A B C D E F G H I J K L M N O P Q R S T U,
	A B C D E F G H I J K L M N O P Q R S T U V,
}


use super::*;

pub unsafe trait VecTrait<T>: std::ops::DerefMut<Target = [T]> + ExtendExt<T> {}

unsafe impl<T> VecTrait<T> for Vec<T> {}


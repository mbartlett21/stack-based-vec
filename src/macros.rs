/// # Examples
///
/// - Creates a [`crate::ArrayVec`] containing a given list of elements:
///
/// ```
/// use stack_based_vec::*;
///
/// let v: ArrayVec<i32, 5> = array_vec![1, 2, 3];
/// assert_eq!(v.capacity(), 5);
/// assert_eq!(v, [1, 2, 3]);
/// ```
///
/// - Creates a [`crate::ArrayVec`] from a given element and size:
///
/// ```
/// use stack_based_vec::*;
///
/// let v: ArrayVec<i32, 5> = array_vec![1; 3];
/// assert_eq!(v.capacity(), 5);
/// assert_eq!(v, [1, 1, 1]);
/// ```
#[macro_export]
macro_rules! array_vec {
    () => ($crate::ArrayVec::new());
    ($elem:expr; $n:expr) => ($crate::ArrayVec::from_partial_array([$elem; $n]));
    ($($elem:expr),+ $(,)?) => ($crate::ArrayVec::from_partial_array([$($elem),+]));
}

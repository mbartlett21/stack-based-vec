/// # Examples
///
/// - Create a [`crate::ArrayVec`] containing a given list of elements:
///
/// ```
/// let v: stack_based_vec::ArrayVec<i32, 3> = stack_based_vec::array_vec![1, 2, 3];
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// ```
///
/// - Create a [`crate::ArrayVec`] from a given element and size:
///
/// ```
/// let v: stack_based_vec::ArrayVec<i32, 3> = stack_based_vec::array_vec![1; 3];
/// assert_eq!(v.as_slice(), &[1, 1, 1]);
/// ```
#[macro_export]
macro_rules! array_vec {
    () => (
        $crate::ArrayVec::new()
    );
    ($element:expr; $n:expr) => ({
        let mut v = $crate::ArrayVec::new();
        for _ in 0..$n.min(v.capacity()) {
            let _ = v.push($element.clone());
        }
        v
    });
    ($($x:expr),+ $(,)?) => ({
        let mut v = $crate::ArrayVec::new();
        $(
            let _ = v.push($x);
        )+
        v
    });
}

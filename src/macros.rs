/// # Examples
///
/// - Creates a [`crate::ArrayVec`] containing a given list of elements:
///
/// ```
/// let v: stack_based_vec::ArrayVec<i32, 5> = stack_based_vec::array_vec![1, 2, 3];
/// assert_eq!(v.capacity(), 5);
/// assert_eq!(v.as_slice(), &[1, 2, 3]);
/// ```
///
/// - Creates a [`crate::ArrayVec`] from a given element and size:
///
/// ```
/// let v: stack_based_vec::ArrayVec<i32, 5> = stack_based_vec::array_vec![1i32; 3];
/// assert_eq!(v.capacity(), 5);
/// assert_eq!(v.as_slice(), &[1, 1, 1]);
/// ```
#[macro_export]
macro_rules! array_vec {
    () => (
        $crate::ArrayVec::new()
    );
    ($element:expr; $n:expr) => ({
        let mut v = $crate::ArrayVec::new();
        let upper_bound = if $n > v.capacity() {
            v.capacity()
        }
        else {
            $n
        };
        for _ in 0..upper_bound {
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

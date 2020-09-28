#![feature(min_const_generics)]

mod array_vec_error;

pub use array_vec_error::ArrayVecError;
use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt,
    iter::IntoIterator,
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Index, IndexMut, RangeBounds},
    ptr,
    slice::{self, Iter, IterMut, SliceIndex},
};

pub struct ArrayVec<T, const N: usize> {
    data: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    // Constructors

    pub unsafe fn from_raw_parts(_ptr: *mut T, _len: usize) -> Self {
        todo!()
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let _: ArrayVec<i32, 2> = ArrayVec::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }

    // Methods

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut _
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const _
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// v.push(1);
    /// assert_eq!(v.as_slice(), &[1]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.capacity(), 2);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn dedup(&mut self) {
        todo!()
    }

    pub fn dedup_by<F>(&mut self, _same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        todo!()
    }

    pub fn dedup_by_key<F, K>(&mut self, _key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        todo!()
    }

    pub fn drain<R>(&mut self, _range: R)
    where
        R: RangeBounds<usize>,
    {
        todo!()
    }

    pub fn extend_from_cloneable_slice(&mut self, _other: &[T])
    where
        T: Clone,
    {
        todo!()
    }

    pub fn extend_from_copyable_slice(&mut self, _other: &[T])
    where
        T: Copy,
    {
        todo!()
    }

    pub fn insert(&mut self, _idx: usize, _element: T) {
        todo!()
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.is_empty(), true);
    /// v.push(1);
    /// assert_eq!(v.is_empty(), false);
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.len(), 0);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn pop(&mut self) -> Option<T> {
        todo!()
    }

    pub fn push(&mut self, element: T) {
        self.try_push(element).unwrap()
    }

    pub fn remove(&mut self, _idx: usize) -> T {
        todo!()
    }

    pub fn retain<F>(&mut self, _f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        todo!()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    pub fn splice<R, I>(&mut self, _range: R, _replace_with: I)
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<usize>,
    {
        todo!()
    }

    pub fn split_off(&mut self, _at: usize) -> Self {
        todo!()
    }

    pub fn swap_remove(&mut self, _idx: usize) -> T {
        todo!()
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    pub fn try_extend_from_cloneable_slice(&mut self, _other: &[T]) -> Result<(), ArrayVecError>
    where
        T: Clone,
    {
        todo!()
    }

    pub fn try_extend_from_copyable_slice(&mut self, _other: &[T]) -> Result<(), ArrayVecError>
    where
        T: Copy,
    {
        todo!()
    }

    pub fn try_insert(&mut self, _idx: usize, _element: T) -> Result<(), ArrayVecError> {
        todo!()
    }

    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.try_push(1).is_ok());
    /// assert_eq!(v[0], 1);
    /// assert!(v.try_push(2).is_ok());
    /// assert_eq!(v[1], 2);
    /// assert!(v.try_push(3).is_err());
    /// ```
    pub fn try_push(&mut self, element: T) -> Result<(), ArrayVecError> {
        if self.len >= N {
            return Err(ArrayVecError::CapacityOverflow);
        }
        unsafe {
            ptr::write(self.as_mut_ptr().add(self.len), element);
        }
        self.len += 1;
        Ok(())
    }
}

impl<T, const N: usize> AsRef<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> AsMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> Borrow<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> BorrowMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> Default for ArrayVec<T, N>
where
    T: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for ArrayVec<T, N> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for ArrayVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Eq for ArrayVec<T, N> where T: Eq {}

impl<T, const N: usize> From<[T; N]> for ArrayVec<T, N> {
    #[inline]
    fn from(from: [T; N]) -> Self {
        Self {
            len: from.len(),
            data: MaybeUninit::new(from),
        }
    }
}

impl<I, T, const N: usize> Index<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;

    #[inline]
    fn index(&self, idx: I) -> &Self::Output {
        &self.as_slice()[idx]
    }
}

impl<I, T, const N: usize> IndexMut<I> for ArrayVec<T, N>
where
    I: SliceIndex<[T]>,
{
    #[inline]
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[idx]
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a ArrayVec<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ArrayVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> Ord for ArrayVec<T, N>
where
    T: Ord,
{
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T, const N: usize> PartialEq for ArrayVec<T, N>
where
    T: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T, const N: usize> PartialOrd for ArrayVec<T, N>
where
    T: PartialOrd,
{
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T, const N: usize> fmt::Debug for ArrayVec<T, N>
where
    T: fmt::Debug,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

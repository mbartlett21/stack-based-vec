#![feature(min_const_generics, slice_partition_dedup)]

use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt,
    iter::IntoIterator,
    mem::MaybeUninit,
    ops::{Bound, Deref, DerefMut, Index, IndexMut, RangeBounds},
    ptr,
    slice::{self, Iter, IterMut, SliceIndex},
};

pub struct ArrayVec<T, const N: usize> {
    data: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    // Constructors

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<i32, 2> = ArrayVec::from_array([1, 2]);
    /// assert_eq!(v.len(), 2);
    /// ```
    #[inline]
    pub fn from_array(array: [T; N]) -> Self {
        Self {
            len: array.len(),
            data: MaybeUninit::new(array),
        }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<i32, 2> = ArrayVec::from_array_and_len([1, 2], 1);
    /// assert_eq!(v.len(), 1);
    /// ```
    #[inline]
    pub fn from_array_and_len(array: [T; N], len: usize) -> Self {
        Self {
            len: len.min(array.len()),
            data: MaybeUninit::new(array),
        }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.len(), 0);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }

    // Methods

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<usize, 2> = ArrayVec::from_array([1, 2]);
    /// let ptr = v.as_mut_ptr();
    ///
    /// unsafe {
    ///     for i in 0..v.len() {
    ///         *ptr.add(i) = i;
    ///     }
    /// }
    ///
    /// assert_eq!(v.as_slice(), &[0, 1]);
    /// ```
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut _
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<usize, 2> = ArrayVec::from_array([1, 2]);
    /// assert_eq!(v.as_mut_slice(), &mut [1, 2]);
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<usize, 2> = ArrayVec::from_array([1, 2]);
    /// let ptr = v.as_ptr();
    ///
    /// unsafe {
    ///     for i in 0..v.len() {
    ///         assert_eq!(*ptr.add(i), i.saturating_add(1));
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const _
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<usize, 2> = ArrayVec::from_array([1, 2]);
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.capacity(), 2);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::from_array([1, 2]);
    /// v.clear();
    /// assert_eq!(v.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 5> = ArrayVec::from_array([1, 2, 2, 3, 2]);
    /// v.dedup();
    /// assert_eq!(v.as_slice(), &[1, 2, 3, 2]);
    /// ```
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(|a, b| a == b)
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 5> = ArrayVec::from_array([10, 20, 21, 30, 20]);
    /// v.dedup_by_key(|i| *i / 10);
    /// assert_eq!(v.as_slice(), &[10, 20, 30, 20]);
    /// ```
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        let len = {
            let (dedup, _) = self.as_mut_slice().partition_dedup_by(same_bucket);
            dedup.len()
        };
        self.truncate(len);
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 5> = ArrayVec::from_array([1, 2, 3, 4, 5]);
    /// let keep = [false, true, true, false, true];
    /// let mut i = 0;
    /// v.retain(|_| (keep[i], i += 1).0);
    /// assert_eq!(v.as_slice(), &[2, 3, 5]);
    /// ```
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    pub fn drain<R>(&mut self, range: R) -> Option<()>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len();
        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(&n) => n + 1,
            Bound::Excluded(&n) => n,
            Bound::Unbounded => len,
        };

        if start > end || end > len {
            return None;
        }

        todo!()
    }

    /// * Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.extend_from_cloneable_slice(&[1, 2]).is_ok());
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.extend_from_cloneable_slice(&[1, 2, 3]).unwrap_err(), &[3]);
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    pub fn extend_from_cloneable_slice<'a>(&mut self, other: &'a [T]) -> Result<(), &'a [T]>
    where
        T: Clone,
    {
        let remaining_capacity = self.remaining_capacity();
        let mut f = |len| {
            for element in other[..len].iter().cloned() {
                let _ = self.push(element);
            }
        };
        if other.len() > remaining_capacity {
            f(remaining_capacity);
            Err(&other[remaining_capacity..])
        } else {
            f(other.len());
            Ok(())
        }
    }

    /// * Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.extend_from_copyable_slice(&[1, 2]).is_ok());
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.extend_from_copyable_slice(&[1, 2, 3]).unwrap_err(), &[3]);
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    pub fn extend_from_copyable_slice<'a>(&mut self, other: &'a [T]) -> Result<(), &'a [T]>
    where
        T: Copy,
    {
        let remaining_capacity = self.remaining_capacity();
        let mut f = |len| unsafe {
            let dst = self.as_mut_ptr().add(self.len());
            ptr::copy_nonoverlapping(other.as_ptr(), dst, len);
            self.increase_len(len);
        };
        if other.len() > remaining_capacity {
            f(remaining_capacity);
            Err(&other[remaining_capacity..])
        } else {
            f(other.len());
            Ok(())
        }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// let _ = v.push(2);
    /// // Index is out of bounds
    /// assert!(v.insert(10, 4).is_err());
    /// // Ok
    /// assert!(v.insert(0, 4).is_ok());
    /// assert_eq!(v.len(), 2);
    /// // Full capacity
    /// assert!(v.insert(0, 6).is_err());
    /// ```
    pub fn insert(&mut self, idx: usize, element: T) -> Result<(), T> {
        if idx > self.len() || self.remaining_capacity() == 0 {
            return Err(element);
        }
        unsafe {
            let ptr: *mut _ = self.as_mut_ptr().add(idx);
            ptr::copy(ptr, ptr.add(1), self.len().saturating_sub(idx));
            ptr::write(ptr, element);
        }
        self.increase_len(1);
        Ok(())
    }

    /// * Example
    ///
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

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.len(), 0);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 3> = ArrayVec::from_array([1, 2, 3]);
    /// assert_eq!(v.pop().unwrap(), 3);
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                self.decrease_len(1);
                Some(ptr::read(self.as_ptr().add(self.len())))
            }
        }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.push(1).is_ok());
    /// assert_eq!(v[0], 1);
    /// assert!(v.push(2).is_ok());
    /// assert_eq!(v[1], 2);
    /// assert!(v.push(3).is_err());
    /// ```
    #[inline]
    pub fn push(&mut self, element: T) -> Result<(), T> {
        if self.len >= N {
            return Err(element);
        }
        unsafe {
            ptr::write(self.as_mut_ptr().add(self.len), element);
        }
        self.increase_len(1);
        Ok(())
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 3> = ArrayVec::from_array([1, 2, 3]);
    /// assert!(v.remove(10).is_none());
    /// assert_eq!(v.remove(0).unwrap(), 1);
    /// assert_eq!(v.as_slice(), &[2, 3]);
    /// ```
    pub fn remove(&mut self, idx: usize) -> Option<T> {
        let len = self.len();
        if idx >= len {
            return None;
        }
        unsafe {
            let ptr = self.as_mut_ptr().add(idx);
            let ret = ptr::read(ptr);
            ptr::copy(ptr.offset(1), ptr, len - idx - 1);
            self.decrease_len(1);
            Some(ret)
        }
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 5> = ArrayVec::from_array([1, 2, 3, 4, 5]);
    /// let keep = [false, true, true, false, true];
    /// let mut i = 0;
    /// v.retain(|_| (keep[i], i += 1).0);
    /// assert_eq!(v.as_slice(), &[2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;
            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            self.truncate(len - del);
        }
    }

    pub fn splice<R, I>(&mut self, _range: R, _replace_with: I)
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<usize>,
    {
        todo!()
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 3> = ArrayVec::from_array([1, 2, 3]);
    /// let v2 = v.split_off(1).unwrap();
    /// assert_eq!(v.as_slice(), &[1]);
    /// assert_eq!(v2.as_slice(), &[2, 3]);
    /// ```
    pub fn split_off(&mut self, at: usize) -> Option<Self> {
        let len = self.len();
        if at > len {
            return None;
        }
        let mut other = Self::new();
        unsafe {
            self.len = at;
            other.len = len - at;
            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len());
        }
        Some(other)
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::from_array([1, 2]);
    /// assert!(v.swap_remove(10).is_none());
    /// assert_eq!(v.swap_remove(0).unwrap(), 1);
    /// assert_eq!(v.get(0).unwrap(), &2);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn swap_remove(&mut self, idx: usize) -> Option<T> {
        let len = self.len();
        if idx >= len {
            return None;
        }
        unsafe {
            let last = ptr::read(self.as_ptr().add(len - 1));
            let hole = self.as_mut_ptr().add(idx);
            self.decrease_len(1);
            Some(ptr::replace(hole, last))
        }
    }

    /// * Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::from_array([1, 2]);
    /// v.truncate(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn truncate(&mut self, len: usize) {
        unsafe {
            if len < self.len() {
                ptr::drop_in_place(&mut self[len..]);
                self.len = len;
            }
        }
    }

    #[inline]
    fn decrease_len(&mut self, n: usize) {
        self.len = self.len.saturating_sub(n);
    }

    #[inline]
    fn increase_len(&mut self, n: usize) {
        self.len = self.len.saturating_add(n);
    }

    #[inline]
    const fn remaining_capacity(&self) -> usize {
        self.capacity().saturating_sub(self.len())
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

impl<T, const N: usize> Clone for ArrayVec<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut v = Self::new();
        let _ = v.extend_from_cloneable_slice(self.as_slice());
        v
    }
}

impl<T, const N: usize> Copy for ArrayVec<T, N> where T: Copy {}

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

impl<T, const N: usize> Extend<T> for ArrayVec<T, N> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let remaining_capacity = self.remaining_capacity();
        for element in iter.into_iter().take(remaining_capacity) {
            let _ = self.push(element);
        }
    }
}

impl<T, const N: usize> From<[T; N]> for ArrayVec<T, N> {
    #[inline]
    fn from(from: [T; N]) -> Self {
        Self::from_array(from)
    }
}

impl<T, const N: usize> From<([T; N], usize)> for ArrayVec<T, N> {
    #[inline]
    fn from(from: ([T; N], usize)) -> Self {
        Self::from_array_and_len(from.0, from.1)
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
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

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

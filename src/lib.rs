// #![allow(trivial_casts, missing_docs)]
#![feature(
    const_deref,
    const_fn_trait_bound,
    const_for,
    const_intrinsic_copy,
    const_maybe_uninit_as_ptr,
    // const_maybe_uninit_assume_init,
    const_maybe_uninit_write,
    const_mut_refs,
    // const_panic,
    const_precise_live_drops,
    const_ptr_offset,
    const_ptr_read,
    const_ptr_write,
    const_raw_ptr_deref,
    const_slice_from_raw_parts,
    const_trait_impl,
    const_try,
    exact_size_is_empty,
    maybe_uninit_extra,
    // maybe_uninit_ref,
    maybe_uninit_uninit_array,
    option_result_unwrap_unchecked,
    slice_partition_dedup,
    trusted_len,
)]

mod drain;
mod macros;
mod splice;

use core::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt,
    hint::unreachable_unchecked,
    iter::IntoIterator,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Bound, Deref, DerefMut, RangeBounds, Index, IndexMut},
    ptr::{self, NonNull},
    slice::{self, Iter, IterMut, SliceIndex},
};

pub use drain::Drain;
pub use splice::Splice;

// #[doc(hidden)]
// pub fn __assert_copy<T: Copy>(_: T) {}

pub struct ArrayVec<T, const N: usize> {
    data: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    // Constructors

    /// Constructs a filled `ArrayVec` from an array.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v = ArrayVec::from_array([1, 2]);
    ///
    /// assert_eq!(v.len(), 2);
    /// assert_eq!(v.capacity(), 2);
    /// ```
    #[inline]
    pub const fn from_array(array: [T; N]) -> Self {
        Self {
            data: MaybeUninit::new(array),
            len: N,
        }
    }

    /// Constructs a partially filled `ArrayVec` from an array.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<i32, 10> = ArrayVec::from_partial_array([1, 2]);
    ///
    /// assert_eq!(v.len(), 2);
    /// ```
    #[inline]
    pub const fn from_partial_array<const M: usize>(array: [T; M]) -> Self {
        if M > N {
            panic!("cannot make ArrayVec from larger array");
        }

        let mut s = Self::new();

        s.len = M;

        let mut array = ManuallyDrop::new(array);

        // SAFETY: Both pointers are valid
        unsafe { ptr::copy_nonoverlapping(array.as_mut_ptr(), s.data.as_mut_ptr() as *mut _, M) }

        s
    }

    pub fn make_filled_array<const M: usize>(&mut self) -> Option<[T; M]> {
        if self.len >= M {
            self.len -= M;

            let ptr = self.as_mut_ptr();

            // SAFETY: These bytes get overwritten by the copy below.
            let v = unsafe { (ptr as *mut [T; M]).read() };

            // SAFETY: We are just copying it back, like in remove(usize).
            // we have already decreased the len
            unsafe { ptr::copy::<T>(ptr.add(M), ptr, self.len) };

            Some(v)
        } else {
            None
        }
    }

    /// Constructs a new, empty `ArrayVec`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// const EMPTY_VEC: ArrayVec<i32, 10> = ArrayVec::new();
    ///
    /// assert_eq!(EMPTY_VEC.len(), 0);
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            data: MaybeUninit::uninit(),
            len: 0,
        }
    }

    /// Returns the length of the inner buffer of the `ArrayVec`.
    ///
    /// Just checking the const parameter is preferred.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v: ArrayVec<i32, 2> = ArrayVec::new();
    ///
    /// assert_eq!(v.capacity(), 2);
    /// ```
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Returns the number of elements in the vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert_eq!(v.len(), 0);
    ///
    /// v.push(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the vector has no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.is_empty());
    ///
    /// v.push(1);
    /// assert!(!v.is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends an element on the back of the vector.
    ///
    /// Panics if the vector is full.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    ///
    /// v.push(1);
    /// assert_eq!(v, [1]);
    ///
    /// v.push(2);
    /// assert_eq!(v, [1, 2]);
    ///
    /// assert!(v.try_push(3).is_err());
    /// ```
    #[inline]
    pub const fn push(&mut self, element: T) {
        if self.len == N {
            panic!("capacity overflow")
        } else {
            unsafe { self.as_mut_ptr().add(self.len).write(element) };

            self.len += 1;
        }
    }

    pub const fn try_push(&mut self, element: T) -> Result<(), T> {
        if self.len == N {
            Err(element)
        } else {
            unsafe { self.as_mut_ptr().add(self.len).write(element) };

            self.len += 1;

            Ok(())
        }
    }

    /// Pops an element from the back of the vector and and returns it, or [`None`]
    /// if it is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 3]);
    /// assert_eq!(v.pop(), Some(3));
    /// assert_eq!(v, [1, 2]);
    /// ```
    #[inline]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.len -= 1;

            Some(unsafe { self.as_mut_ptr().add(self.len).read() })
        }
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2]);
    /// let ptr = v.as_mut_ptr();
    ///
    /// for i in 0..v.len() {
    ///     unsafe { *ptr.add(i) = i };
    /// }
    ///
    /// assert_eq!(v.as_slice(), &[0, 1]);
    /// ```
    #[inline]
    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr() as *mut _
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v = ArrayVec::from_array([1, 2]);
    /// assert_eq!(v.as_mut_slice(), &mut [1, 2]);
    /// ```
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { &mut *ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len) }
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let v = ArrayVec::from_array([1, 2]);
    /// let ptr = v.as_ptr();
    ///
    /// for i in 0..v.len() {
    ///     assert_eq!(unsafe { *ptr.add(i) }, i + 1);
    /// }
    /// ```
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.data.as_ptr() as *const _
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let v = ArrayVec::from_array([1, 2]);
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        unsafe { &*ptr::slice_from_raw_parts(self.as_ptr(), self.len) }
    }

    // Can't be const because of Drop

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2]);
    /// v.clear();
    /// assert_eq!(v.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// Deduplicates equal consequent elements
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 2, 3, 2]);
    /// v.dedup();
    /// assert_eq!(v.as_slice(), &[1, 2, 3, 2]);
    /// ```
    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        self.dedup_by(|a, b| a == b)
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::{ArrayVec, array_vec};
    ///
    /// let mut v: ArrayVec<i32, 10> = array_vec![10, 20, 21, 30, 20, 24];
    ///
    /// v.dedup_by(|x, y| (*x - *y).abs() < 5);
    ///
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

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([10, 20, 21, 30, 20]);
    ///
    /// v.dedup_by_key(|i| *i / 10);
    ///
    /// assert_eq!(v.as_slice(), &[10, 20, 30, 20]);
    /// ```
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq<K>,
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v = ArrayVec::from_array([1, 2, 3]);
    /// {
    ///     let mut iter = v.drain(1..).unwrap();
    ///     assert_eq!(iter.next().unwrap(), 2);
    ///     assert_eq!(iter.next().unwrap(), 3);
    /// }
    /// assert_eq!(v.as_slice(), &[1]);
    /// v.drain(..);
    /// assert_eq!(v.as_slice(), &[]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Option<Drain<'_, T, N>>
    where
        R: RangeBounds<usize>,
    {
        let len = self.len;

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

        // set self.vec length's to start, to be safe in case Drain is leaked
        self.len = start;

        // Use the borrow in the IterMut to indicate borrowing behavior of the
        // whole Drain iterator (like &mut T).
        let range_slice = unsafe { slice::from_raw_parts(self.as_ptr().add(start), end - start) };

        Some(Drain {
            tail_start: end,
            tail_len: len - end,
            iter: range_slice.iter(),
            vec: NonNull::from(self),
        })
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.extend_from_cloneable_slice(&[1, 2]).is_ok());
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// assert!(v.extend_from_cloneable_slice(&[1, 2, 3]).is_err());
    /// assert_eq!(v.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub fn extend_from_cloneable_slice<'a>(&mut self, other: &'a [T]) -> Result<(), &'a [T]>
    where
        T: Clone,
    {
        let other_len = other.len();
        let remaining_capacity = self.remaining_capacity();
        macro_rules! do_clone {
            ($additional_len:expr) => {
                for elem in other[0..$additional_len].iter().cloned() {
                    unsafe { self.try_push(elem).unwrap_unchecked() };
                }
            };
        }
        if other_len > remaining_capacity {
            do_clone!(remaining_capacity);
            Err(&other[remaining_capacity..])
        } else {
            do_clone!(other_len);
            Ok(())
        }
    }

    /// # Examples
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
    #[inline]
    pub const fn extend_from_copyable_slice<'a>(&mut self, other: &'a [T]) -> Result<(), &'a [T]>
    where
        T: Copy,
    {
        let remaining_capacity = self.remaining_capacity();

        let dst = unsafe { self.as_mut_ptr().add(self.len) };

        if other.len() > remaining_capacity {
            unsafe { ptr::copy_nonoverlapping(other.as_ptr(), dst, remaining_capacity) };

            self.len = N;

            // We use slice_from_raw_parts so that it is const.
            Err(unsafe {
                &*ptr::slice_from_raw_parts(
                    &other[remaining_capacity],
                    other.len() - remaining_capacity,
                )
            })
            // Err(&other[remaining_capacity..])
        } else {
            unsafe { ptr::copy_nonoverlapping(other.as_ptr(), dst, other.len()) };

            self.len += other.len();
            Ok(())
        }
    }

    /// # Example
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    /// let mut v: ArrayVec<i32, 2> = ArrayVec::new();
    /// v.push(2);
    ///
    /// // Index is out of bounds
    /// assert!(v.insert(10, 4).is_err());
    ///
    /// // Ok
    /// assert!(v.insert(0, 4).is_ok());
    /// assert_eq!(v.len(), 2);
    ///
    /// // Full capacity
    /// assert!(v.insert(0, 6).is_err());
    /// ```
    pub const fn insert(&mut self, idx: usize, element: T) -> Result<(), T> {
        if idx > self.len || self.len == N {
            Err(element)
        } else {
            let ptr = unsafe { self.as_mut_ptr().add(idx) };
            unsafe { ptr.copy_to(ptr.add(1), self.len - idx) };
            unsafe { ptr.write(element) };
            self.len += 1;

            Ok(())
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 3]);
    /// assert_eq!(v.remove(10), None);
    /// assert_eq!(v.remove(0), Some(1));
    /// assert_eq!(v.as_slice(), &[2, 3]);
    /// ```
    pub const fn remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.len {
            None
        } else {
            let ptr = unsafe { self.as_mut_ptr().add(idx) };
            let result = unsafe { ptr.read() };
            unsafe { ptr.copy_from(ptr.add(1), self.len - idx - 1) };
            self.len -= 1;

            Some(result)
        }
    }

    // Can't be const because of drop and trait methods

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 3, 4, 5]);
    /// v.retain(|e| *e % 2 == 1);
    /// assert_eq!(v.as_slice(), &[1, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let len = self.len;
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

    // non-const because of trait

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 3]);
    /// {
    ///     let mut iter = v.splice(..2, [7, 8].iter().copied()).unwrap();
    ///     assert_eq!(iter.next().unwrap(), 1);
    ///     assert_eq!(iter.next().unwrap(), 2);
    /// }
    /// assert_eq!(v.as_slice(), &[7, 8, 3]);
    /// ```
    #[inline]
    pub fn splice<I, R>(
        &mut self,
        range: R,
        replace_with: I,
    ) -> Option<Splice<'_, I::IntoIter, N>>
    where
        I: IntoIterator<Item = T>,
        R: RangeBounds<usize>,
    {
        Some(Splice {
            drain: self.drain(range)?,
            replace_with: replace_with.into_iter(),
        })
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2, 3]);
    /// let v2 = v.split_off(1).unwrap();
    /// assert_eq!(v.as_slice(), &[1]);
    /// assert_eq!(v2.as_slice(), &[2, 3]);
    /// ```
    pub const fn split_off(&mut self, at: usize) -> Option<Self> {
        let len = self.len;
        if at > len {
            None
        } else {
            let mut other_arr_vec = Self::new();

            self.len = at;
            other_arr_vec.len = len - at;

            unsafe {
                self.as_ptr()
                    .add(at)
                    .copy_to_nonoverlapping(other_arr_vec.as_mut_ptr(), len - at);
            }

            Some(other_arr_vec)
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2]);
    /// assert!(v.swap_remove(10).is_none());
    /// assert_eq!(v.swap_remove(0), Some(1));
    ///
    /// assert_eq!(v[0], 2);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub const fn swap_remove(&mut self, idx: usize) -> Option<T> {
        if idx >= self.len {
            return None;
        }

        self.len -= 1;

        if self.len == 0 {
            let v = unsafe { self.as_ptr().read() };
            Some(v)
        } else {
            let last = unsafe { self.as_ptr().add(self.len).read() };
            let hole = unsafe { self.as_mut_ptr().add(idx) };

            let v = unsafe { hole.read() };
            unsafe { hole.write(last) };

            Some(v)
        }
    }

    /// # Examples
    ///
    /// ```rust
    /// use stack_based_vec::ArrayVec;
    ///
    /// let mut v = ArrayVec::from_array([1, 2]);
    /// v.truncate(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        if len > self.len {
            return;
        }
        let remaining_len = self.len - len;
        let s = unsafe { ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len) };
        self.len = len;
        unsafe { ptr::drop_in_place(s) };
    }

    #[inline]
    const fn remaining_capacity(&self) -> usize {
        self.capacity() - self.len
    }

    /// # Safety
    ///
    /// At least `len` elements *must* be initialized.
    /// `len` cannot exceed `N`.
    #[inline]
    pub const unsafe fn set_len(&mut self, len: usize) {
        debug_assert!(len <= N, "len out of bounds");

        self.len = len;
    }
}

impl<T, const N: usize> const AsRef<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> const AsMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> const Borrow<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> const BorrowMut<[T]> for ArrayVec<T, N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T, const N: usize> Clone for ArrayVec<T, N>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        let mut v = Self::new();
        match v.extend_from_cloneable_slice(self.as_slice()) {
            Ok(()) => {}
            Err(_) => unsafe { unreachable_unchecked() },
        }
        v
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.clear();

        match self.extend_from_cloneable_slice(source.as_slice()) {
            Ok(()) => {}
            Err(_) => unsafe { unreachable_unchecked() },
        }
    }
}

impl<T, const N: usize> const Copy for ArrayVec<T, N> where T: ~const Copy {}

impl<T, const N: usize> const Default for ArrayVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> const Deref for ArrayVec<T, N> {
    type Target = [T];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> const DerefMut for ArrayVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Eq for ArrayVec<T, N> where T: Eq {}

impl<T, const N: usize> Extend<T> for ArrayVec<T, N> {
    #[inline]
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

impl<T, const N: usize> const From<[T; N]> for ArrayVec<T, N> {
    #[inline]
    fn from(from: [T; N]) -> Self {
        Self::from_array(from)
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

macro_rules! __impl_slice_eq1 {
    ([$($vars:tt)*] $lhs:ty, $rhs:ty $(where $ty:ty: $bound:ident)*) => {
        impl<T, U, $($vars)* const N: usize> PartialEq<$rhs> for $lhs
        where
            T: PartialEq<U>,
            $($ty: $bound)*
        {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool { self[..] == other[..] }
            #[allow(clippy::partialeq_ne_impl)]
            #[inline]
            fn ne(&self, other: &$rhs) -> bool { self[..] != other[..] }
        }
    }
}

__impl_slice_eq1! { [const O: usize,] ArrayVec<T, N>, ArrayVec<U, O> }
__impl_slice_eq1! { [] ArrayVec<T, N>, &[U] }
__impl_slice_eq1! { [] ArrayVec<T, N>, &mut [U] }
__impl_slice_eq1! { [] &[T], ArrayVec<U, N> }
__impl_slice_eq1! { [] &mut [T], ArrayVec<U, N> }
__impl_slice_eq1! { [] ArrayVec<T, N>, [U] }
__impl_slice_eq1! { [] [T], ArrayVec<U, N>  }
// __impl_slice_eq1! { [] Cow<'_, [T]>, ArrayVec<U, N> where T: Clone }
// __impl_slice_eq1! { [] Cow<'_, [T]>, &[U] where T: Clone }
// __impl_slice_eq1! { [] Cow<'_, [T]>, &mut [U] where T: Clone }
__impl_slice_eq1! { [const O: usize,] ArrayVec<T, N>, [U; O] }
__impl_slice_eq1! { [const O: usize,] ArrayVec<T, N>, &[U; O] }

// impl<T, U, const N: usize, const O: usize> const PartialEq<ArrayVec<U, O>> for ArrayVec<T, N>
// where
//     T: ~const PartialEq<U>,
// {
//     #[inline]
//     fn eq(&self, other: &ArrayVec<U, O>) -> bool {
//         self.as_slice() == other.as_slice()
//     }

//     #[inline]
//     fn ne(&self, other: &ArrayVec<U, O>) -> bool {
//         self.as_slice() != other.as_slice()
//     }
// }

// impl<T, const N: usize, const O: usize> PartialEq<[T; O]> for ArrayVec<T, N>
// where
//     T: PartialEq,
// {
//     #[inline]
//     fn eq(&self, other: &[T; O]) -> bool {
//         self[..] == other[..]
//     }

//     #[inline]
//     fn ne(&self, other: &[T; O]) -> bool {
//         self[..] != other[..]
//     }
// }

// impl<T, const N: usize> PartialEq<[T]> for ArrayVec<T, N>
// where
//     T: PartialEq,
// {
//     #[inline]
//     fn eq(&self, other: &[T]) -> bool {
//         self[..] == other[..]
//     }

//     #[inline]
//     fn ne(&self, other: &[T]) -> bool {
//         self[..] != other[..]
//     }
// }

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

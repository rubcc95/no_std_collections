use core::{borrow::*, cmp::Ordering, fmt, hash::*, mem::MaybeUninit, ops::*, slice::SliceIndex};

use crate::traits::*;

#[derive(Clone, Debug)]
pub struct StackVec<T, const N: usize> {
    buff: [T; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    const IS_ZST: bool = core::mem::size_of::<T>() == 0;
    const UNINIT_ITEM: T = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    const UNINIT_ARRAY: [T; N] = [Self::UNINIT_ITEM; N];

    #[inline]
    pub const fn new() -> Self {
        Self {
            buff: Self::UNINIT_ARRAY,
            len: 0,
        }
    }

    unsafe fn from_slice_copy_unchecked(slice: &[T]) -> Self {
        let len = slice.len();
        if Self::capacity() < len {
            panic!("Capacity overflow")
        } else {
            let mut buff = Self::UNINIT_ARRAY;
            unsafe {
                buff.as_mut_ptr()
                    .copy_from_nonoverlapping(slice.as_ptr(), len)
            };
            Self { buff, len }
        }
    }

    #[inline]
    pub const fn capacity() -> usize {
        if Self::IS_ZST {
            usize::MAX
        } else {
            N
        }
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    #[inline]
    pub unsafe fn split_off_unchecked(&mut self, at: usize) -> Self {
        Vec::split_off_unchecked(self, at)
    }

    #[inline]
    pub fn split_off(&mut self, at: usize) -> Self {
        Vec::split_off(self, at)
    }

    #[inline]
    pub unsafe fn push_unchecked(&mut self, item: T) {
        Vec::push_unchecked(self, item);
    }

    #[inline]
    pub fn push(&mut self, item: T) {
        Vec::push(self, item)
    }

    #[inline]
    pub unsafe fn append_unchecked<S: SliceOwner<Item = T>>(&mut self, other: S) {
        Vec::append_unchecked(self, other);
    }

    #[inline]
    pub fn append<S: SliceOwner<Item = T>>(&mut self, other: S) {
        Vec::append(self, other)
    }

    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        Vec::pop_unchecked(self)
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        Vec::pop(self)
    }

    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        Vec::swap_remove(self, index)
    }

    #[inline]
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        Vec::swap_remove_unchecked(self, index)
    }

    #[inline]
    pub unsafe fn remove_unchecked(&mut self, index: usize) -> T {
        Vec::remove_unchecked(self, index)
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> T {
        Vec::remove(self, index)
    }

    #[inline]
    pub fn clear(&mut self) {
        Vec::clear(self);
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        Vec::truncate(self, len);
    }

    #[inline]
    pub unsafe fn truncate_unchecked(&mut self, len: usize) {
        Vec::truncate_unchecked(self, len);
    }

    #[inline]
    pub unsafe fn insert_unchecked(&mut self, index: usize, element: T) {
        Vec::insert_unchecked(self, index, element);
    }

    #[inline]
    pub fn insert(&mut self, index: usize, element: T) {
        Vec::insert(self, index, element);
    }

    #[inline]
    pub fn drain<R>(&mut self, range: R) -> vec::Drain<Self>
    where
        R: RangeBounds<usize>,
    {
        Vec::drain(self, range)
    }

    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
        Self: Sized,
    {
        Vec::retain(self, f)
    }

    #[inline]
    pub fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        Vec::retain_mut(self, f)
    }

    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        Vec::dedup_by_key(self, key);
    }

    #[inline]
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        Vec::dedup_by(self, same_bucket);
    }

    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T)
    where
        T: Clone,
    {
        Vec::resize(self, new_len, value);
    }

    #[inline]
    pub fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> T,
    {
        Vec::resize_with(self, new_len, f);
    }

    #[inline]
    pub fn leak<'a>(self) -> &'a mut [T] {
        Vec::leak(self)
    }

    #[inline]
    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        Vec::spare_capacity_mut(self)
    }

    #[inline]
    pub unsafe fn extend_from_slice_unchecked(&mut self, slice: &[T])
    where
        T: Clone,
    {
        Vec::extend_from_slice_unchecked(self, slice);
    }

    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T])
    where
        T: Clone,
    {
        Vec::extend_from_slice(self, slice);
    }

    #[inline]
    pub fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        T: Clone,
    {
        Vec::extend_from_within(self, src);
    }
}

impl<T, const N: usize> AsMut<StackVec<T, N>> for StackVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut StackVec<T, N> {
        self
    }
}
impl<T, const N: usize> AsMut<[T]> for StackVec<T, N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}
impl<T, const N: usize> AsRef<StackVec<T, N>> for StackVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &StackVec<T, N> {
        self
    }
}

impl<T, const N: usize> AsRef<[T]> for StackVec<T, N> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> Borrow<[T]> for StackVec<T, N> {
    #[inline]
    fn borrow(&self) -> &[T] {
        self
    }
}

impl<T, const N: usize> BorrowMut<[T]> for StackVec<T, N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for IntoIter<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, const N: usize> Default for StackVec<T, N> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for StackVec<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for StackVec<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<'a, T: Copy + 'a, const N: usize> Extend<&'a T> for StackVec<T, N> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for &item in iter {
            self.push(item)
        }
    }
}

impl<T, const N: usize> Extend<T> for StackVec<T, N> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item);
        }
    }
}

impl<T, const N: usize> From<[T; N]> for StackVec<T, N> {
    #[inline]
    fn from(value: [T; N]) -> Self {
        Self {
            buff: value,
            len: N,
        }
    }
}

impl<T: Clone, const N: usize> From<&[T; N]> for StackVec<T, N> {
    #[inline]
    fn from(value: &[T; N]) -> Self {
        Self {
            buff: value.clone(),
            len: N,
        }
    }
}

impl<T: Clone, const N: usize> From<&[T]> for StackVec<T, N> {
    fn from(value: &[T]) -> Self {
        unsafe { Self::from_slice_copy_unchecked(value) }
    }
}

impl<T: Clone, const N: usize> From<&mut [T; N]> for StackVec<T, N> {
    #[inline]
    fn from(value: &mut [T; N]) -> Self {
        <Self as From<&[T; N]>>::from(value)
    }
}

impl<T: Clone, const N: usize> From<&mut [T]> for StackVec<T, N> {
    #[inline]
    fn from(value: &mut [T]) -> Self {
        unsafe { Self::from_slice_copy_unchecked(value) }
    }
}

impl<const N: usize> From<&str> for StackVec<u8, N> {
    #[inline]
    fn from(s: &str) -> StackVec<u8, N> {
        unsafe { Self::from_slice_copy_unchecked(s.as_bytes()) }
    }
}

#[cfg(feature = "std")]
impl<T, const N: usize> From<Box<[T; N]>> for StackVec<T, N> {
    fn from(value: Box<[T; N]>) -> Self {
        let mut buff = Self::UNINIT_ARRAY;
        unsafe { buff.as_mut_ptr()
            .copy_from_nonoverlapping(value.as_ptr(), N) };
        core::mem::forget(value);
        Self { buff, len: N }
    }
}

#[cfg(feature = "std")]
impl<T, const N: usize> From<Box<[T]>> for StackVec<T, N> {
    fn from(value: Box<[T]>) -> Self {
        let this = unsafe { Self::from_slice_copy_unchecked(&*value) };
        core::mem::forget(value);
        this
    }
}

#[cfg(feature = "std")]
impl<T, const N: usize> From<std::vec::Vec<T>> for StackVec<T, N> {
    fn from(value: std::vec::Vec<T>) -> Self {
        let this = unsafe { Self::from_slice_copy_unchecked(&value) };
        core::mem::forget(value);
        this
    }
}

// #[cfg(feature = "std")]
// impl<const N: usize> From<std::ffi::CString> for StackVec<u8, N> {
//     #[inline]
//     fn from(s: std::ffi::CString) -> StackVec<u8, N> {
//         let this = unsafe { Self::from_slice_copy_unchecked(s.as_bytes()) };
//         core::mem::forget(s);
//         this
//     }
// }

// #[cfg(feature = "std")]
// impl<const N: usize> From<String> for StackVec<u8, N> {
//     #[inline]
//     fn from(s: String) -> StackVec<u8, N> {
//         let this = unsafe { Self::from_slice_copy_unchecked(s.as_bytes()) };
//         core::mem::forget(s);
//         this
//     }
// }

// #[cfg(feature = "std")]
// impl<'a, T: Clone, const N: usize> From<&'a StackVec<T, N>> for std::borrow::Cow<'a, [T]> {
//     #[inline]
//     fn from(v: &'a StackVec<T, N>) -> std::borrow::Cow<'a, [T]> {
//         std::borrow::Cow::Borrowed(v.as_slice())
//     }
// }

impl<T, const N: usize> FromIterator<T> for StackVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut this = Self::new();
        for item in iter {
            Vec::push(&mut this, item);
        }
        this
    }
}

impl<T: Hash, const N: usize> Hash for StackVec<T, N> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(self.as_slice(), state)
    }
}

impl<T, const N: usize> IntoIterator for StackVec<T, N> {
    type Item = T;

    type IntoIter = IntoIter<T, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            array: self.buff,
            start: 0,
            end: self.len,
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a StackVec<T, N> {
    type Item = &'a T;

    type IntoIter = core::slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut StackVec<T, N> {
    type Item = &'a mut T;

    type IntoIter = core::slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize, I: SliceIndex<[T]>> Index<I> for StackVec<T, N> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(self.as_slice(), index)
    }
}

impl<T, const N: usize, I: SliceIndex<[T]>> IndexMut<I> for StackVec<T, N> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(self.as_mut_slice(), index)
    }
}

impl<T: Eq, const N: usize> Eq for StackVec<T, N> {}

impl<T: Ord, const N: usize> Ord for StackVec<T, N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: PartialOrd, const N1: usize, const N2: usize> PartialOrd<StackVec<T, N1>>
    for StackVec<T, N2>
{
    fn partial_cmp(&self, other: &StackVec<T, N1>) -> Option<Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<StackVec<U, N1>>
    for StackVec<T, N2>
{
    fn eq(&self, other: &StackVec<U, N1>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<[U; N1]> for StackVec<T, N2> {
    fn eq(&self, other: &[U; N1]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<[U]> for StackVec<T, N2> {
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<&[U; N1]> for StackVec<T, N2> {
    fn eq(&self, other: &&[U; N1]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<&[U]> for StackVec<T, N2> {
    fn eq(&self, other: &&[U]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<&mut [U; N1]>
    for StackVec<T, N2>
{
    fn eq(&self, other: &&mut [U; N1]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<&mut [U]> for StackVec<T, N2> {
    fn eq(&self, other: &&mut [U]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<StackVec<U, N2>> for [T; N1] {
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<StackVec<U, N2>> for [T] {
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<StackVec<U, N2>> for &[T; N1] {
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        (*self).eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<StackVec<U, N2>> for &[T] {
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        (*self).eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<StackVec<U, N2>>
    for &mut [T; N1]
{
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        (*self as &[T; N1]).eq(other.as_slice())
    }
}

impl<T: PartialEq<U>, U, const N2: usize> PartialEq<StackVec<U, N2>> for &mut [T] {
    fn eq(&self, other: &StackVec<U, N2>) -> bool {
        (*self as &[T]).eq(other.as_slice())
    }
}
// impl<T: PartialEq<U>, U, const N1: usize, const N2: usize> PartialEq<&[U; N1]> for StackVec<T, N2>
// {
//     fn eq(&self, other: &&[U; N1]) -> bool {
//         let slice = self.as_slice();
//         slice.partial_
//     }
// }

unsafe impl<T, const N: usize> SliceOwner for StackVec<T, N> {
    type Item = T;
    
    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn as_ptr(&self) -> *const T {
        self.buff.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.buff.as_mut_ptr()
    }
}

unsafe impl<T, const N: usize> Vec for StackVec<T, N> {
    #[inline]
    fn capacity(&self) -> usize {
        Self::capacity()
    }

    #[inline]
    unsafe fn set_len(&mut self, new_len: usize) {
        self.set_len(new_len)
    }

    #[inline]
    fn reserve(&mut self, _: usize) {
        panic!("Can not reserve in a StackVec!")
    }

    unsafe fn split_off_unchecked(&mut self, at: usize) -> Self {
        let other_len = self.len().unchecked_sub(at);
        let mut other = Self {
            buff: Self::UNINIT_ARRAY,
            len: other_len,
        };

        unsafe {
            self.set_len(at);
            self.as_ptr()
                .add(at)
                .copy_to_nonoverlapping(other.as_mut_ptr(), other_len);
        }
        other
    }
}

pub struct IntoIter<T, const N: usize> {
    array: [T; N],
    start: usize,
    end: usize,
}

impl<T, const N: usize> IntoIter<T, N> {
    #[inline(always)]
    pub fn new(vec: StackVec<T, N>) -> Self {
        vec.into_iter()
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.array.as_ptr().add(self.start), self.len()) }
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.array.as_mut_ptr().add(self.start), self.len())
        }
    }
}

impl<T, const N: usize> Default for IntoIter<T, N> {
    #[inline]
    fn default() -> Self {
        Self {
            array: StackVec::UNINIT_ARRAY,
            start: 0,
            end: 0,
        }
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.end > self.start {
            let curr = self.start;
            self.start = unsafe { curr.unchecked_add(1) };
            Some(unsafe { self.array.as_ptr().add(curr).read() })
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn last(mut self) -> Option<T> {
        self.next_back()
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<T> {
        if self.end > self.start {
            let curr = self.end;
            self.end = unsafe { curr.unchecked_sub(1) };
            Some(unsafe { self.array.as_ptr().add(curr).read() })
        } else {
            None
        }
    }
}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { core::ptr::drop_in_place(self.as_mut_slice()) }
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    #[inline(always)]
    fn len(&self) -> usize {
        unsafe { self.end.unchecked_sub(self.start) }
    }
}

impl<T, const N: usize> core::iter::FusedIterator for IntoIter<T, N> {}

impl<T: Clone, const N: usize> Clone for IntoIter<T, N> {
    fn clone(&self) -> Self {
        let mut new = StackVec::<T, N>::UNINIT_ARRAY;
        let ptr = new.as_mut_ptr();
        for index in self.start..self.end {
            unsafe {
                ptr.add(index)
                    .write(self.array.get_unchecked(index).clone())
            };
        }
        Self {
            array: new,
            start: self.start,
            end: self.end,
        }
    }
}

mod drain;
mod ops;

pub use drain::Drain;
pub use ops::Ops;

use super::SliceOwner;
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Bound, Range, RangeBounds},
    ptr,
};

pub unsafe trait Vec: SliceOwner {
    fn capacity(&self) -> usize;

    unsafe fn set_len(&mut self, new_len: usize);

    fn reserve(&mut self, additional: usize);

    unsafe fn split_off_unchecked(&mut self, at: usize) -> Self
    where
        Self: Sized;

    fn split_off(&mut self, at: usize) -> Self
    where
        Self: Sized,
    {
        #[cold]        
        #[track_caller]
        fn assert_failed(at: usize, len: usize) -> ! {
            panic!("`at` split index (is {at}) should be <= len (is {len})");
        }

        if at > self.len() {
            assert_failed(at, self.len());
        }
        unsafe { self.split_off_unchecked(at) }
    }

    unsafe fn push_unchecked(&mut self, item: Self::Item) {
        self.as_mut_ptr().add(self.len()).write(item);
        self.set_len(self.len().unchecked_add(1));
    }

    fn push(&mut self, item: Self::Item) {
        self.reserve(1);
        unsafe { self.push_unchecked(item) };
    }

    unsafe fn append_unchecked<S: SliceOwner<Item = Self::Item>>(&mut self, other: S)
    where
        Self: Sized,
    {
        extend_vec_with_raw_parts_unchecked(self, other.as_ptr(), other.len());
        core::mem::forget(other);
    }

    fn append<S: SliceOwner<Item = Self::Item>>(&mut self, other: S)
    where
        Self: Sized,
    {
        self.reserve(other.len());
        {
            unsafe { self.append_unchecked(other) }
        };
    }

    unsafe fn pop_unchecked(&mut self) -> Self::Item {
        self.set_len(self.len().unchecked_sub(1));
        self.as_mut_ptr().add(self.len()).read()
    }

    fn pop(&mut self) -> Option<Self::Item> {
        if self.len() > 0 {
            Some(unsafe { self.pop_unchecked() })
        } else {
            None
        }
    }

    fn swap_remove(&mut self, index: usize) -> Self::Item {
        #[cold]        
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("swap_remove index (is {index}) should be < len (is {len})");
        }
        if index >= self.len() {
            assert_failed(index, self.len());
        }
        unsafe { self.swap_remove_unchecked(index) }
    }

    unsafe fn swap_remove_unchecked(&mut self, index: usize) -> Self::Item {
        let value = core::ptr::read(self.as_ptr().add(index));
        let base_ptr = self.as_mut_ptr();
        let new_len = self.len().unchecked_sub(1);
        base_ptr.add(new_len).copy_from(base_ptr.add(index), 1);
        self.set_len(new_len);
        value
    }

    unsafe fn remove_unchecked(&mut self, index: usize) -> Self::Item {
        let new_len = self.len().unchecked_sub(1);
        self.set_len(new_len);

        // the place we are taking from.
        let ptr = self.as_mut_ptr().add(index);
        let res = ptr.read();
        ptr.copy_from(ptr.add(1), new_len.unchecked_sub(index));
        res
    }

    fn remove(&mut self, index: usize) -> Self::Item {        
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("removal index (is {index}) should be < len (is {len})")
        }

        if index >= self.len() {
            assert_failed(index, self.len());
        }
        unsafe { self.remove_unchecked(index) }
    }

    fn clear(&mut self) {
        let elems = self.as_mut_slice() as *mut [Self::Item];
        unsafe {
            self.set_len(0);
            core::ptr::drop_in_place(elems);
        }
    }

    fn truncate(&mut self, len: usize) {
        if len > self.len() {
            return;
        }
        unsafe { self.truncate_unchecked(len) };
    }

    unsafe fn truncate_unchecked(&mut self, len: usize) {
        let remaining_len = self.len().unchecked_sub(len);
        let s = core::ptr::slice_from_raw_parts_mut(self.as_mut_ptr().add(len), remaining_len);
        self.set_len(len);
        core::ptr::drop_in_place(s);
    }

    unsafe fn insert_unchecked(&mut self, index: usize, element: Self::Item) {
        let p = self.as_mut_ptr().add(index);
        p.copy_to(p.add(1), self.len().unchecked_sub(index));
        p.write(element);
        self.set_len(self.len().unchecked_add(1));
    }

    fn insert(&mut self, index: usize, element: Self::Item) {
        #[cold]        
        #[track_caller]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("insertion index (is {index}) should be <= len (is {len})");
        }

        let len = self.len();

        if index > len {
            assert_failed(index, len);
        }

        self.reserve(1);

        unsafe {
            let p = self.as_mut_ptr().add(index);
            if index < len {
                p.copy_to(p.add(1), len.unchecked_sub(index));
            }
            p.write(element);

            self.set_len(len + 1);
        }
    }

    fn drain<R>(&mut self, range: R) -> Drain<Self>
    where
        R: RangeBounds<usize>,
        Self: Sized,
    {
        let Range { start, end } = slice_range(range, self.len());
        drain::Drain {
            tail_start: end,
            tail_len: unsafe { self.len().unchecked_sub(end) },
            iter: unsafe {
                core::slice::from_raw_parts(self.as_ptr().add(start), end - start).iter()
            },
            vec: ptr::NonNull::from(self),
        }
    }

    fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Self::Item) -> bool,
        Self: Sized,
    {
        self.retain_mut(|i| f(i));
    }

    fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Self::Item) -> bool,
        Self: Sized,
    {
        let original_len = self.len();
        unsafe { self.set_len(0) };
        struct BackshiftOnDrop<'a, V: Vec + ?Sized> {
            v: &'a mut V,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<V: Vec + ?Sized> Drop for BackshiftOnDrop<'_, V> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    unsafe {
                        self.v.as_ptr().add(self.processed_len).copy_to(
                            self.v
                                .as_mut_ptr()
                                .add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop {
            v: self,
            processed_len: 0,
            deleted_cnt: 0,
            original_len,
        };

        fn process_loop<F, V, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, V>,
        ) where
            V: Vec + ?Sized,
            F: FnMut(&mut V::Item) -> bool,
        {
            while g.processed_len != original_len {
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    unsafe { core::ptr::drop_in_place(cur) };
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        hole_slot.copy_from_nonoverlapping(cur, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        process_loop::<F, Self, false>(original_len, &mut f, &mut g);
        process_loop::<F, Self, true>(original_len, &mut f, &mut g);
        drop(g);
    }

    fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut Self::Item) -> K,
        K: PartialEq,
        Self: Sized,
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    fn dedup_by<F>(&mut self, mut same_bucket: F)
    where
        F: FnMut(&mut Self::Item, &mut Self::Item) -> bool,
        Self: Sized,
    {
        let len = self.len();
        if len <= 1 {
            return;
        }
        let mut first_duplicate_idx: usize = 1;
        let start = self.as_mut_ptr();
        while first_duplicate_idx != len {
            let found_duplicate = unsafe {
                let prev = start.add(first_duplicate_idx.wrapping_sub(1));
                let current = start.add(first_duplicate_idx);
                same_bucket(&mut *current, &mut *prev)
            };
            if found_duplicate {
                break;
            }
            first_duplicate_idx += 1;
        }
        if first_duplicate_idx == len {
            return;
        }
        struct FillGapOnDrop<'a, V: Vec + ?Sized> {
            read: usize,
            write: usize,
            vec: &'a mut V,
        }

        impl<'a, V: Vec + ?Sized> Drop for FillGapOnDrop<'a, V> {
            fn drop(&mut self) {
                unsafe {
                    let ptr = self.vec.as_mut_ptr();
                    let len = self.vec.len();
                    let items_left = len.wrapping_sub(self.read);
                    let dropped_ptr = ptr.add(self.write);
                    let valid_ptr = ptr.add(self.read);
                    valid_ptr.copy_to(dropped_ptr, items_left);
                    let dropped = self.read.wrapping_sub(self.write);
                    self.vec.set_len(len - dropped);
                }
            }
        }
        let mut gap = FillGapOnDrop {
            read: first_duplicate_idx + 1,
            write: first_duplicate_idx,
            vec: self,
        };
        unsafe {
            core::ptr::drop_in_place(start.add(first_duplicate_idx));
        }
        unsafe {
            while gap.read < len {
                let read_ptr = start.add(gap.read);
                let prev_ptr = start.add(gap.write.wrapping_sub(1));
                let found_duplicate = same_bucket(&mut *read_ptr, &mut *prev_ptr);
                if found_duplicate {
                    gap.read += 1;
                    core::ptr::drop_in_place(read_ptr);
                } else {
                    let write_ptr = start.add(gap.write);
                    read_ptr.copy_to_nonoverlapping(write_ptr, 1);
                    gap.write += 1;
                    gap.read += 1;
                }
            }
            gap.vec.set_len(gap.write);
            core::mem::forget(gap);
        }
    }

    fn resize(&mut self, new_len: usize, value: Self::Item)
    where
        Self::Item: Clone,
    {
        if new_len > self.len() {
            let last = unsafe { new_len.unchecked_sub(1) };
            for i in self.len()..last {
                unsafe { self.as_mut_ptr().add(i).write(value.clone()) };
            }
            unsafe { self.as_mut_ptr().add(last).write(value) };
            unsafe { self.set_len(new_len) };
        } else {
            unsafe { self.truncate_unchecked(new_len) };
        }
    }

    fn resize_with<F>(&mut self, new_len: usize, mut f: F)
    where
        F: FnMut() -> Self::Item,
        Self: Sized,
    {
        if new_len > self.len() {
            for i in self.len()..new_len {
                unsafe { self.as_mut_ptr().add(i).write(f()) };
            }
            unsafe { self.set_len(new_len) };
        } else {
            unsafe { self.truncate_unchecked(new_len) };
        }
    }

    fn leak<'a>(self) -> &'a mut [Self::Item]
    where
        Self: Sized,
    {
        let mut this = ManuallyDrop::new(self);
        unsafe { core::slice::from_raw_parts_mut(this.as_mut_ptr(), this.len()) }
    }

    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.as_mut_ptr().add(self.len()) as *mut MaybeUninit<Self::Item>,
                self.capacity().unchecked_sub(self.len()),
            )
        }
    }

    unsafe fn extend_from_slice_unchecked(&mut self, slice: &[Self::Item])
    where
        Self::Item: Clone,
    {
        extend_vec_with_raw_parts_unchecked(self, slice.as_ptr(), slice.len());
    }

    fn extend_from_slice(&mut self, slice: &[Self::Item])
    where
        Self::Item: Clone,
    {
        self.reserve(slice.len());
        unsafe { self.extend_from_slice_unchecked(slice) };
    }

    fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        Self: Sized,
        Self::Item: Clone,
    {
        let range = slice_range(src, self.len());
        let len = range.len();
        self.reserve(len);
        unsafe { extend_vec_with_raw_parts_unchecked(self, self.as_ptr().add(range.start), len) };
    }
}

#[cfg(feature = "std")]
unsafe impl<T> Vec for std::vec::Vec<T> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.capacity()
    }

    #[inline(always)]
    unsafe fn set_len(&mut self, new_len: usize) {
        self.set_len(new_len);
    }

    #[inline(always)]
    fn reserve(&mut self, additional: usize) {
        self.reserve(additional);
    }

    unsafe fn split_off_unchecked(&mut self, at: usize) -> Self {
        let other_len = self.len().unchecked_sub(at);
        let mut other = Self::with_capacity(other_len);

        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            self.as_ptr()
                .add(at)
                .copy_to_nonoverlapping(other.as_mut_ptr(), other_len);
        }
        other
    }

    #[inline(always)]
    fn push(&mut self, item: Self::Item) {
        self.push(item)
    }

    #[inline(always)]
    fn split_off(&mut self, at: usize) -> Self
    where
        Self: Sized,
    {
        self.split_off(at)
    }

    #[inline(always)]
    fn pop(&mut self) -> Option<Self::Item> {
        self.pop()
    }

    #[inline(always)]
    fn swap_remove(&mut self, index: usize) -> Self::Item {
        self.swap_remove(index)
    }

    #[inline(always)]
    fn remove(&mut self, index: usize) -> Self::Item {
        self.remove(index)
    }

    #[inline(always)]
    fn clear(&mut self) {
        self.clear();
    }

    #[inline(always)]
    fn truncate(&mut self, len: usize) {
        self.truncate(len);
    }

    #[inline(always)]
    fn insert(&mut self, index: usize, element: Self::Item) {
        self.insert(index, element);
    }

    #[inline(always)]
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Item) -> bool,
    {
        self.retain(f);
    }

    #[inline(always)]
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut Self::Item) -> bool,
    {
        self.retain_mut(f);
    }

    #[inline(always)]
    fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut Self::Item) -> K,
        K: PartialEq,
    {
        self.dedup_by_key(key);
    }

    #[inline(always)]
    fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut Self::Item, &mut Self::Item) -> bool,
    {
        self.dedup_by(same_bucket);
    }

    #[inline(always)]
    fn resize(&mut self, new_len: usize, value: Self::Item)
    where
        Self::Item: Clone,
    {
        self.resize(new_len, value);
    }

    #[inline(always)]
    fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> Self::Item,
    {
        self.resize_with(new_len, f);
    }

    fn leak<'a>(self) -> &'a mut [Self::Item]
    where
        Self: Sized,
    {
        self.leak()
    }

    #[inline(always)]
    fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<Self::Item>] {
        self.spare_capacity_mut()
    }

    #[inline(always)]
    fn extend_from_slice(&mut self, other: &[Self::Item])
    where
        Self::Item: Clone,
    {
        self.extend_from_slice(other);
    }

    #[inline(always)]
    fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        Self::Item: Clone,
    {
        self.extend_from_within(src);
    }
}

pub trait VecUnsized: Vec {
    unsafe fn split_off_unchecked(&mut self, at: usize) -> Self;

    fn split_off(&mut self, at: usize) -> Self;

    unsafe fn append_unchecked<S: SliceOwner<Item = Self::Item>>(&mut self, other: S);

    fn append<S: SliceOwner<Item = Self::Item>>(&mut self, other: S);

    fn drain<R>(&mut self, range: R) -> Drain<Self>
    where
        R: RangeBounds<usize>;

    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Item) -> bool;

    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut Self::Item) -> bool;

    fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut Self::Item) -> K,
        K: PartialEq;

    fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut Self::Item, &mut Self::Item) -> bool,
        Self: Sized;

    fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> Self::Item;

    fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        Self::Item: Clone;
}

impl<T: Vec> VecUnsized for T {
    #[inline(always)]
    unsafe fn split_off_unchecked(&mut self, at: usize) -> Self {
        self.split_off_unchecked(at)
    }
    #[inline(always)]
    fn split_off(&mut self, at: usize) -> Self {
        self.split_off(at)
    }

    #[inline(always)]
    unsafe fn append_unchecked<S: SliceOwner<Item = Self::Item>>(&mut self, other: S) {
        Vec::append_unchecked(self, other);
    }
    #[inline(always)]
    fn append<S: SliceOwner<Item = Self::Item>>(&mut self, other: S) {
        Vec::append(self, other);
    }
    #[inline(always)]
    fn drain<R>(&mut self, range: R) -> Drain<Self>
    where
        R: RangeBounds<usize>,
    {
        Vec::drain(self, range)
    }
    #[inline(always)]
    fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Self::Item) -> bool,
    {
        Vec::retain(self, f);
    }
    #[inline(always)]
    fn retain_mut<F>(&mut self, f: F)
    where
        F: FnMut(&mut Self::Item) -> bool,
    {
        Vec::retain_mut(self, f);
    }
    #[inline(always)]
    fn dedup_by_key<F, K>(&mut self, key: F)
    where
        F: FnMut(&mut Self::Item) -> K,
        K: PartialEq,
    {
        Vec::dedup_by_key(self, key);
    }
    #[inline(always)]
    fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut Self::Item, &mut Self::Item) -> bool,
        Self: Sized,
    {
        Vec::dedup_by(self, same_bucket);
    }

    #[inline(always)]
    fn resize_with<F>(&mut self, new_len: usize, f: F)
    where
        F: FnMut() -> Self::Item,
    {
        Vec::resize_with(self, new_len, f);
    }
    #[inline(always)]
    fn extend_from_within<R>(&mut self, src: R)
    where
        R: RangeBounds<usize>,
        Self::Item: Clone,
    {
        Vec::extend_from_within(self, src);
    }
}

unsafe fn extend_vec_with_raw_parts_unchecked<V: Vec + ?Sized>(
    this: &mut V,
    src: *const V::Item,
    count: usize,
) {
    src.copy_to_nonoverlapping(this.as_mut_ptr(), this.len());
    this.set_len(this.len().unchecked_add(count));
}

fn slice_range<R>(range: R, len: usize) -> Range<usize>
where
    R: RangeBounds<usize>,
{
    let start = match range.start_bound() {
        Bound::Included(&start) => start,
        Bound::Excluded(start) => start
            .checked_add(1)
            .unwrap_or_else(|| slice_start_index_overflow_fail()),
        Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
        Bound::Included(end) => end
            .checked_add(1)
            .unwrap_or_else(|| slice_end_index_overflow_fail()),
        Bound::Excluded(&end) => end,
        Bound::Unbounded => len,
    };

    if start > end {
        slice_index_order_fail(start, end);
    }
    if end > len {
        slice_end_index_len_fail(end, len);
    }

    Range { start, end }
}

#[inline(never)]
#[track_caller]
const fn slice_start_index_overflow_fail() -> ! {
    panic!("attempted to index slice from after maximum usize");
}


#[inline(never)]
#[track_caller]
const fn slice_end_index_overflow_fail() -> ! {
    panic!("attempted to index slice up to maximum usize");
}

#[inline(never)]
#[track_caller]
fn slice_index_order_fail(index: usize, end: usize) -> ! {
    panic!("slice index starts at {index} but ends at {end}");
}
#[inline(never)]
#[track_caller]
fn slice_end_index_len_fail(index: usize, len: usize) -> ! {
    panic!("range end index {index} out of range for slice of length {len}");
}
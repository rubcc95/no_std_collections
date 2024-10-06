use super::*;

use core::{fmt, slice, ptr, mem};

pub struct Drain<'a, T: VecUnsized + ?Sized> {
    /// Index of tail to preserve
    pub(super) tail_start: usize,
    /// Length of tail
    pub(super) tail_len: usize,
    /// Current remaining range to remove
    pub(super) iter: slice::Iter<'a, T::Item>,
    pub(super) vec: ptr::NonNull<T>,
}

impl<'a, T: VecUnsized + ?Sized> Drain<'a, T>{    
    const IS_ZST: bool = mem::size_of::<T::Item>() == 0;
}

impl<T: VecUnsized<Item: fmt::Debug> + ?Sized> fmt::Debug for Drain<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}

impl<'a, T: VecUnsized + ?Sized> Drain<'a, T> {
    pub fn as_slice(&self) -> &[T::Item] {
        self.iter.as_slice()
    }
}

impl<'a, T: VecUnsized + ?Sized> AsRef<[T::Item]> for Drain<'a, T> {
    fn as_ref(&self) -> &[T::Item] {
        self.as_slice()
    }
}

unsafe impl<T: VecUnsized<Item: Sync> + ?Sized> Sync for Drain<'_, T> {}

unsafe impl<T: VecUnsized<Item: Send> + ?Sized> Send for Drain<'_, T> {}

impl<V: VecUnsized + ?Sized> Iterator for Drain<'_, V> {
    type Item = V::Item;

    #[inline]
    fn next(&mut self) -> Option<V::Item> {
        self.iter
            .next()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<T: VecUnsized + ?Sized> DoubleEndedIterator for Drain<'_, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T::Item> {
        self.iter
            .next_back()
            .map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<T: VecUnsized + ?Sized> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        /// Moves back the un-`Drain`ed elements to restore the original `Vec`.
        struct DropGuard<'r, 'a, T: VecUnsized + ?Sized>(&'r mut Drain<'a, T>);

        impl<'r, 'a, T: VecUnsized + ?Sized> Drop for DropGuard<'r, 'a, T> {
            fn drop(&mut self) {
                if self.0.tail_len > 0 {
                    unsafe {
                        let source_vec = self.0.vec.as_mut();
                        // memmove back untouched tail, update to new length
                        let start = source_vec.len();
                        let tail = self.0.tail_start;
                        if tail != start {
                            let src = source_vec.as_ptr().add(tail);
                            let dst = source_vec.as_mut_ptr().add(start);
                            ptr::copy(src, dst, self.0.tail_len);
                        }
                        source_vec.set_len(start + self.0.tail_len);
                    }
                }
            }
        }

        let iter = mem::take(&mut self.iter);
        let drop_len = iter.len();

        let mut vec = self.vec;

        if Self::IS_ZST {
            unsafe {
                let vec = vec.as_mut();
                let old_len = vec.len();
                vec.set_len(old_len + drop_len + self.tail_len);
                vec.truncate(old_len + self.tail_len);
            }

            return;
        }

        let _guard = DropGuard(self);

        if drop_len == 0 {
            return;
        }

        let drop_ptr = iter.as_slice().as_ptr();

        unsafe {
            let vec_ptr = vec.as_mut().as_mut_ptr();
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                vec_ptr.add(drop_ptr.offset_from(vec_ptr) as usize),
                drop_len,
            ))
        }
    }
}

impl<T: VecUnsized + ?Sized> ExactSizeIterator for Drain<'_, T> {
    fn len(&self) -> usize {
        self.tail_len
    }
}

impl<T: VecUnsized + ?Sized> core::iter::FusedIterator for Drain<'_, T> {}

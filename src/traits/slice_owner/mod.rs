pub unsafe trait SliceOwner {
    type Item;

    fn len(&self) -> usize;

    fn as_ptr(&self) -> *const Self::Item;

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.as_ptr() as *mut Self::Item
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        unsafe { core::slice::from_raw_parts(self.as_ptr(), self.len()) }
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), self.len()) }
    }
}

unsafe impl<T, const N: usize> SliceOwner for [T; N] {
    type Item = T;
    #[inline]
    fn as_ptr(&self) -> *const Self::Item {
        self as *const T
    }

    #[inline]
    fn len(&self) -> usize {
        N
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self as *mut T
    }

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        self.as_slice()
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        self.as_mut_slice()
    }
}

#[cfg(feature = "std")]
unsafe impl<T> SliceOwner for Vec<T> {
    type Item = T;
    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn as_ptr(&self) -> *const Self::Item {
        self.as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        self.as_mut_ptr()
    }
}

#[cfg(feature = "std")]
unsafe impl<T> SliceOwner for Box<[T]> {    
    type Item = T;

    #[inline]
    fn len(&self) -> usize {
        (&**self).len()
    }

    #[inline]
    fn as_ptr(&self) -> *const Self::Item {
        (&**self).as_ptr()
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut Self::Item {
        (&mut **self).as_mut_ptr()
    }

    fn as_slice(&self) -> &[Self::Item] {
        &**self
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Item] {
        &mut **self
    }
}
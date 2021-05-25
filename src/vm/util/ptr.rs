use std::ptr;

pub struct NonNull<T>(*mut T);

impl<T> NonNull<T> {
    pub fn new(ptr: *mut T) -> Option<NonNull<T>> {
        if ptr.is_null() {
            None
        } else {
            Some(NonNull(ptr))
        }
    }

    /// Can only be used as placeholder
    pub unsafe fn new_null() -> NonNull<T> {
        NonNull(ptr::null_mut())
    }

    pub unsafe fn as_ref<'a, 'b>(&'a self) -> &'b T {
        &*self.0
    }

    pub unsafe fn as_mut<'a, 'b>(&'a self) -> &'b mut T {
        &mut *self.0
    }

    pub fn as_ptr(&self) -> *mut T {
        self.0
    }
}

impl<T> PartialEq for NonNull<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for NonNull<T> {}

impl<T> Clone for NonNull<T> {
    fn clone(&self) -> Self {
        NonNull(self.0)
    }
}

impl<T> Copy for NonNull<T> {}

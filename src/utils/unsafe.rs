use std::ops::{Deref, DerefMut};

/// A wrapper that unsafely implements `Sync`.
pub struct UnsafeSync<T> {
    value: T,
}

impl<T> UnsafeSync<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> Deref for UnsafeSync<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for UnsafeSync<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

unsafe impl<T> Send for UnsafeSync<T> {}
unsafe impl<T> Sync for UnsafeSync<T> {}

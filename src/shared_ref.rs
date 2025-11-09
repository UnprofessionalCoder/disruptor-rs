use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct Counter<T> {
    c: T,
    count: AtomicUsize,
}

pub struct SharedRef<T> {
    counter: NonNull<Counter<T>>,
}

unsafe impl<T> Send for SharedRef<T> {}
unsafe impl<T> Sync for SharedRef<T> {}

impl<T> SharedRef<T> {
    pub fn new(c: T) -> Self {
        SharedRef {
            counter: NonNull::from(Box::leak(Box::new(Counter {
                c: c,
                count: AtomicUsize::new(1),
            }))),
        }
    }

    fn counter(&self) -> &Counter<T> {
        unsafe { self.counter.as_ref() }
    }

    fn mut_counter(&mut self) -> &mut Counter<T> {
        unsafe { self.counter.as_mut() }
    }
}

impl<T> Clone for SharedRef<T> {
    fn clone(&self) -> Self {
        self.counter().count.fetch_add(1, Ordering::Relaxed);

        Self {
            counter: self.counter,
        }
    }
}

impl<T> Deref for SharedRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.counter().c
    }
}

impl<T> DerefMut for SharedRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mut_counter().c
    }
}

impl<T> Drop for SharedRef<T> {
    fn drop(&mut self) {
        if self.counter().count.fetch_sub(1, Ordering::AcqRel) == 1 {
            drop(unsafe { Box::from_raw(self.counter.as_ptr()) });
        }
    }
}

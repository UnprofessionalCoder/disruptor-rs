use std::{cell::UnsafeCell, mem::MaybeUninit};

use crate::EventFactory;

pub struct RingBuffer<E> {
    buffer: Box<[UnsafeCell<MaybeUninit<E>>]>,
    mask: usize,
}

unsafe impl<E: Send> Send for RingBuffer<E> {}
unsafe impl<E: Sync> Sync for RingBuffer<E> {}

impl<E> RingBuffer<E> {
    pub fn new<F>(buffer_size: usize, event_factory: F) -> Self
    where
        F: EventFactory<E>,
    {
        assert!(buffer_size > 0, "buffer size must > 0");

        assert!(
            buffer_size.is_power_of_two(),
            "buffer size must be power of two"
        );

        let buffer = (0..buffer_size)
            .map(|_| UnsafeCell::new(MaybeUninit::new(event_factory.new())))
            .collect();

        RingBuffer {
            buffer,
            mask: buffer_size - 1,
        }
    }
}

impl<E> RingBuffer<E> {
    pub fn get(&self, sequence: usize) -> &E {
        let index = sequence & self.mask;
        let data = unsafe { &*(*self.buffer.get_unchecked(index).get()).assume_init_ref() };
        data
    }

    pub fn get_mut(&self, sequence: usize) -> &mut E {
        let index = sequence & self.mask;
        let data = unsafe { &mut *(*self.buffer.get_unchecked(index).get()).assume_init_mut() };
        data
    }
}

impl<E> Drop for RingBuffer<E> {
    fn drop(&mut self) {
        let buffer_size = self.mask + 1;
        for i in 0..buffer_size {
            unsafe { (*(self.buffer.get_unchecked(i).get())).assume_init_drop() };
        }
    }
}

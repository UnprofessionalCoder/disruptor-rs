use std::sync::atomic::{AtomicI64, Ordering};

#[repr(align(64))]
pub struct Sequence {
    value: AtomicI64,
}

impl Sequence {
    pub fn new() -> Self {
        Sequence {
            value: AtomicI64::new(-1),
        }
    }

    pub fn set(&self, new_value: i64) {
        self.value.store(new_value, Ordering::Release);
    }

    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Acquire)
    }

    pub fn compare_exchange_weak(&self, current: i64, new: i64) -> bool {
        self.value
            .compare_exchange_weak(current, new, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }
}

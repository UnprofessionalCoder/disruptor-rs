use std::{sync::Arc, thread};

use crate::{Sequence, WaitStrategy, util};

pub struct BusySpinWaitStrategy;

impl WaitStrategy for BusySpinWaitStrategy {
    fn wait_for<F: Fn() -> bool>(
        &self,
        sequence: i64,
        _cursor: Arc<Sequence>,
        dependent_sequence: Vec<Arc<Sequence>>,
        check_alert: F,
    ) -> Option<i64> {
        let mut available_sequence = util::minimum_sequence(&dependent_sequence);
        while available_sequence < sequence {
            if check_alert() {
                return None;
            }
            available_sequence = util::minimum_sequence(&dependent_sequence);
            std::hint::spin_loop();
        }
        Some(available_sequence)
    }

    fn signal_all_when_blocking(&self) {}
}

pub struct YieldingWaitStrategy;

impl WaitStrategy for YieldingWaitStrategy {
    fn wait_for<F: Fn() -> bool>(
        &self,
        sequence: i64,
        _cursor: Arc<Sequence>,
        dependent_sequence: Vec<Arc<Sequence>>,
        check_alert: F,
    ) -> Option<i64> {
        let mut available_sequence = util::minimum_sequence(&dependent_sequence);
        while available_sequence < sequence {
            if check_alert() {
                return None;
            }
            available_sequence = util::minimum_sequence(&dependent_sequence);
            thread::yield_now();
        }
        Some(available_sequence)
    }

    fn signal_all_when_blocking(&self) {}
}

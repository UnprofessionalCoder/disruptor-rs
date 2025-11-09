use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use crate::{
    EventProcessor, RingBuffer, Runnable, STATE_HALTED, STATE_IDLE, STATE_RUNNING, Sequence,
    SequenceBarrier, WorkHandler,
};

pub struct WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync,
{
    sequence: Arc<Sequence>,
    work_sequence: Arc<Sequence>,
    work_handler: H,
    ringbuffer: Arc<RingBuffer<E>>,
    sequence_barrier: Arc<B>,
    state: AtomicU8,
}

unsafe impl<E, B, H> Send for WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync,
{
}

unsafe impl<E, B, H> Sync for WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync,
{
}

impl<E, B, H> WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync,
{
    pub fn new(
        work_sequence: Arc<Sequence>,
        work_handler: H,
        ringbuffer: Arc<RingBuffer<E>>,
        sequence_barrier: Arc<B>,
    ) -> Self {
        WorkProcessor {
            sequence: Arc::new(Sequence::new()),
            work_sequence,
            work_handler,
            ringbuffer,
            sequence_barrier,
            state: AtomicU8::new(STATE_IDLE),
        }
    }
}

impl<E, B, H> Runnable for WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync,
{
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            if self
                .state
                .compare_exchange(
                    STATE_IDLE,
                    STATE_RUNNING,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                self.sequence_barrier.clear_alert();

                let mut processed_sequence = true;
                let mut cached_available_sequence = -1;
                let mut next_sequence = self.sequence.get();

                while self.state.load(Ordering::Acquire) == STATE_RUNNING {
                    if processed_sequence {
                        processed_sequence = false;
                        loop {
                            next_sequence = self.work_sequence.get() + 1;
                            self.sequence.set(next_sequence);

                            if self
                                .work_sequence
                                .compare_exchange_weak(next_sequence - 1, next_sequence)
                            {
                                break;
                            }
                        }
                    }

                    if cached_available_sequence >= next_sequence {
                        let event = self.ringbuffer.get_mut(next_sequence as usize);
                        self.work_handler.on_event(event);
                        processed_sequence = true;
                    } else {
                        if let Some(s) = self.sequence_barrier.wait_for(next_sequence) {
                            cached_available_sequence = s;
                        } else {
                            break;
                        }
                    }
                }

                self.state.store(STATE_IDLE, Ordering::Release);
            }
        })
    }
}

impl<E, B, H> EventProcessor for WorkProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: WorkHandler<E>,
    E: Send + Sync + 'static,
{
    fn halt(&mut self) {
        self.state.store(STATE_HALTED, Ordering::Release);
        self.sequence_barrier.alert();
    }

    fn sequence(&self) -> Arc<Sequence> {
        self.sequence.clone()
    }

    fn running(self) -> bool {
        self.state.load(Ordering::Acquire) == STATE_RUNNING
    }
}

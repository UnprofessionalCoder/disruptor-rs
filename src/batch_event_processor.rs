use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use crate::{
    EventHandler, EventProcessor, RingBuffer, Runnable, STATE_HALTED, STATE_IDLE, STATE_RUNNING,
    Sequence, SequenceBarrier,
};

pub struct BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
{
    sequence: Arc<Sequence>,
    event_handler: H,
    ringbuffer: Arc<RingBuffer<E>>,
    sequence_barrier: Arc<B>,
    state: AtomicU8,
}

unsafe impl<E, B, H> Send for BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
{
}

unsafe impl<E, B, H> Sync for BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
{
}

impl<E, B, H> BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
{
    pub fn new(event_handler: H, ringbuffer: Arc<RingBuffer<E>>, sequence_barrier: Arc<B>) -> Self {
        BatchEventProcessor {
            sequence: Arc::new(Sequence::new()),
            event_handler,
            ringbuffer,
            sequence_barrier,
            state: AtomicU8::new(STATE_IDLE),
        }
    }

    fn process_events(&mut self) {
        let mut next_sequence = self.sequence.get() + 1;

        let available_sequence = self.sequence_barrier.wait_for(next_sequence);

        if let Some(sequence) = available_sequence {
            while next_sequence <= sequence {
                let event = self.ringbuffer.get_mut(next_sequence as usize);
                self.event_handler
                    .on_event(event, next_sequence, next_sequence == sequence);
                next_sequence += 1;
            }

            self.sequence.set(sequence);
        }
    }
}

impl<E, B, H> Runnable for BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
    E: 'static,
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

                while self.state.load(Ordering::Acquire) == STATE_RUNNING {
                    self.process_events();
                }

                self.state.store(STATE_IDLE, Ordering::Release);
            }
        })
    }
}

impl<E, B, H> EventProcessor for BatchEventProcessor<E, B, H>
where
    B: SequenceBarrier,
    H: EventHandler<E>,
    E: 'static,
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

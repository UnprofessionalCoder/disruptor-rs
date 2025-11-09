use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    thread,
};

use crate::{Producer, Publisher, RingBuffer, Sequence, Sequencer, SequencerAdapter, util};

pub struct SingleProducerSequencer {
    buffer_size: i64,
    next_value: i64,
    cached_value: i64,
    cursor: Arc<Sequence>,
    gating_sequences: Vec<Arc<Sequence>>,
}

unsafe impl Send for SingleProducerSequencer {}

impl SingleProducerSequencer {
    pub fn new(buffer_size: i64) -> Self {
        Self {
            buffer_size,
            next_value: -1,
            cached_value: -1,
            cursor: Arc::new(Sequence::new()),
            gating_sequences: vec![],
        }
    }

    fn min_gating_sequence(&self) -> i64 {
        util::minimum_sequence(&self.gating_sequences)
    }
}

impl Sequencer for SingleProducerSequencer {
    fn next(&mut self, sequence: i64) -> i64 {
        let next_value = self.next_value;

        let next_sequence = next_value + sequence;
        let wrap_point = next_sequence - self.buffer_size;

        let cached_gate = self.cached_value;
        if wrap_point > cached_gate || cached_gate > next_value {
            self.cursor.set(next_value);

            let mut min_sequence = self.min_gating_sequence();
            while wrap_point > min_sequence {
                thread::yield_now();
                min_sequence = self.min_gating_sequence();
            }
            self.cached_value = min_sequence;
        }

        self.next_value = next_sequence;
        next_sequence
    }

    fn publish(&self, sequence: i64) {
        self.cursor.set(sequence);
    }

    fn batch_publish(&self, low: i64, high: i64) {
        self.publish(high);
    }

    fn highest_published(&self, next_sequence: i64, available_sequence: i64) -> i64 {
        available_sequence
    }

    fn available(&self, sequence: i64) -> bool {
        sequence <= self.cursor.get()
    }

    fn cursor(&self) -> Arc<Sequence> {
        self.cursor.clone()
    }

    fn add_gating_sequence(&mut self, gating_sequence: Arc<Sequence>) {
        self.gating_sequences.push(gating_sequence);
    }

    fn buffer_size(&self) -> i64 {
        self.buffer_size
    }
}

pub struct SingleProducer<E, S>
where
    S: Sequencer,
{
    publisher: Publisher<E, S>,
}

unsafe impl<E, S> Send for SingleProducer<E, S> where S: Sequencer {}

impl<E, S> SingleProducer<E, S>
where
    S: Sequencer,
{
    pub fn new(
        sequencer: SequencerAdapter<S>,
        ringbuffer: Arc<RingBuffer<E>>,
    ) -> SingleProducer<E, S> {
        SingleProducer {
            publisher: Publisher::new(sequencer, ringbuffer),
        }
    }
}

impl<E, S> Producer<E, S> for SingleProducer<E, S>
where
    S: Sequencer,
{
    fn mut_sequencer(&mut self) -> &mut S {
        self.publisher.mut_sequencer()
    }
}

impl<E, S> Deref for SingleProducer<E, S>
where
    S: Sequencer,
{
    type Target = Publisher<E, S>;

    fn deref(&self) -> &Self::Target {
        &self.publisher
    }
}

impl<E, S> DerefMut for SingleProducer<E, S>
where
    S: Sequencer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.publisher
    }
}

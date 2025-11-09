use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    thread,
};

use crate::{
    Producer, Publisher, RingBuffer, Sequence, Sequencer, SequencerAdapter, SharedRef, util,
};

pub struct MultiProducerSequencer {
    buffer_size: i64,
    index_mask: i64,
    index_shift: i64,
    cursor: Arc<Sequence>,
    gating_sequence_cache: Sequence,
    gating_sequences: Vec<Arc<Sequence>>,
    available_buffer: Box<[Sequence]>,
}

unsafe impl Send for MultiProducerSequencer {}
unsafe impl Sync for MultiProducerSequencer {}

impl MultiProducerSequencer {
    pub fn new(buffer_size: i64) -> Self {
        let available_buffer = (0..buffer_size)
            .map(|_| Sequence::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let index_shift = (buffer_size as f64).log2() as i64;

        Self {
            buffer_size,
            index_mask: buffer_size - 1,
            index_shift,
            cursor: Arc::new(Sequence::new()),
            gating_sequence_cache: Sequence::new(),
            gating_sequences: vec![],
            available_buffer,
        }
    }

    fn set_available(&self, sequence: i64) {
        let index = (sequence & self.index_mask) as usize;
        let flag = self.calc_availability_flag(sequence);

        let v = unsafe { self.available_buffer.get_unchecked(index) };
        v.set(flag);
    }

    fn calc_availability_flag(&self, sequence: i64) -> i64 {
        sequence >> self.index_shift
    }

    fn calc_index(&self, sequence: i64) -> usize {
        (sequence & self.index_mask) as usize
    }
}

impl Sequencer for MultiProducerSequencer {
    fn next(&mut self, sequence: i64) -> i64 {
        let buffer_size = self.buffer_size;
        loop {
            let current = self.cursor.get();
            let next = current + sequence;

            let wrap_point = next - buffer_size;

            let cached_gating_sequence = self.gating_sequence_cache.get();

            if wrap_point > cached_gating_sequence || cached_gating_sequence > current {
                let gating_sequence = util::minimum_sequence(&self.gating_sequences);

                if wrap_point > gating_sequence {
                    thread::yield_now();
                    continue;
                }

                self.gating_sequence_cache.set(gating_sequence);
            }

            if self.cursor.compare_exchange_weak(current, next) {
                return next;
            }
        }
    }

    fn publish(&self, sequence: i64) {
        self.set_available(sequence);
    }

    fn batch_publish(&self, low: i64, high: i64) {
        for idx in low..=high {
            self.set_available(idx);
        }
    }

    fn highest_published(&self, next_sequence: i64, available_sequence: i64) -> i64 {
        for sequence in next_sequence..=available_sequence {
            if !self.available(sequence) {
                return sequence - 1;
            }
        }

        available_sequence
    }

    fn available(&self, sequence: i64) -> bool {
        let index = self.calc_index(sequence);
        let flag = self.calc_availability_flag(sequence);
        let v = unsafe { self.available_buffer.get_unchecked(index) };
        v.get() == flag
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

pub struct MultiProducer<E, S>
where
    S: Sequencer,
{
    publisher: SharedRef<Publisher<E, S>>,
}

impl<E, S> MultiProducer<E, S>
where
    S: Sequencer,
{
    pub fn new(sequencer: SequencerAdapter<S>, ringbuffer: Arc<RingBuffer<E>>) -> Self {
        MultiProducer {
            publisher: SharedRef::new(Publisher::new(sequencer, ringbuffer)),
        }
    }
}

impl<E, S> Producer<E, S> for MultiProducer<E, S>
where
    S: Sequencer,
{
    fn mut_sequencer(&mut self) -> &mut S {
        self.publisher.mut_sequencer()
    }
}

impl<E, S> Deref for MultiProducer<E, S>
where
    S: Sequencer,
{
    type Target = SharedRef<Publisher<E, S>>;

    fn deref(&self) -> &Self::Target {
        &self.publisher
    }
}

impl<E, S> DerefMut for MultiProducer<E, S>
where
    S: Sequencer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.publisher
    }
}

impl<E, S> Clone for MultiProducer<E, S>
where
    S: Sequencer,
{
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
        }
    }
}

use std::sync::Arc;

use crate::{RingBuffer, Sequencer, SequencerAdapter};

pub struct Publisher<E, S>
where
    S: Sequencer,
{
    sequencer: SequencerAdapter<S>,
    ringbuffer: Arc<RingBuffer<E>>,
}

unsafe impl<E, S> Send for Publisher<E, S> where S: Sequencer {}
unsafe impl<E, S> Sync for Publisher<E, S> where S: Sequencer {}

impl<E, S> Publisher<E, S>
where
    S: Sequencer,
{
    pub fn new(sequencer: SequencerAdapter<S>, ringbuffer: Arc<RingBuffer<E>>) -> Publisher<E, S> {
        Self {
            sequencer,
            ringbuffer,
        }
    }

    pub fn mut_sequencer(&mut self) -> &mut S {
        &mut self.sequencer
    }
}

impl<E, S> Publisher<E, S>
where
    S: Sequencer,
{
    pub fn publish<F>(&mut self, f: F)
    where
        F: Fn(&mut E),
    {
        let sequence = self.sequencer.next(1);
        let event = self.ringbuffer.get_mut(sequence as usize);
        f(event);
        self.sequencer.publish(sequence);
    }
}

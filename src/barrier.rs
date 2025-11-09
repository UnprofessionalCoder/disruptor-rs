use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{Sequence, SequenceBarrier, Sequencer, SequencerAdapter, WaitStrategy};

pub struct ProcessingSequenceBarrier<W, S>
where
    W: WaitStrategy,
    S: Sequencer,
{
    alert: Arc<AtomicBool>,
    wait_strategy: Arc<W>,
    sequencer: SequencerAdapter<S>,
    dependent_sequence: Vec<Arc<Sequence>>,
}

unsafe impl<W, S> Send for ProcessingSequenceBarrier<W, S>
where
    W: WaitStrategy,
    S: Sequencer,
{
}
unsafe impl<W, S> Sync for ProcessingSequenceBarrier<W, S>
where
    W: WaitStrategy,
    S: Sequencer,
{
}

impl<W, S> ProcessingSequenceBarrier<W, S>
where
    W: WaitStrategy,
    S: Sequencer,
{
    pub fn new(
        wait_strategy: Arc<W>,
        sequencer: SequencerAdapter<S>,
        mut dependent_sequence: Vec<Arc<Sequence>>,
    ) -> Self {
        if dependent_sequence.len() == 0 {
            dependent_sequence.push(sequencer.cursor());
        }
        ProcessingSequenceBarrier {
            alert: Arc::new(AtomicBool::new(false)),
            wait_strategy,
            sequencer,
            dependent_sequence,
        }
    }
}

impl<W, S> SequenceBarrier for ProcessingSequenceBarrier<W, S>
where
    W: WaitStrategy + Send + Sync,
    S: Sequencer,
{
    fn wait_for(&self, sequence: i64) -> Option<i64> {
        if let Some(available_sequence) = self.wait_strategy.wait_for(
            sequence,
            self.sequencer.cursor(),
            self.dependent_sequence.clone(),
            || self.alert.load(Ordering::Acquire),
        ) {
            if available_sequence < sequence {
                return Some(available_sequence);
            }

            return Some(
                self.sequencer
                    .highest_published(sequence, available_sequence),
            );
        } else {
            None
        }
    }

    fn alert(&self) {
        self.alert.store(true, Ordering::Release);
        self.wait_strategy.signal_all_when_blocking();
    }

    fn clear_alert(&self) {
        self.alert.store(false, Ordering::Release);
    }
}

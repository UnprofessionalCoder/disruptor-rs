use std::sync::Arc;

use crate::{ConsumerRepository, Executor, RingBuffer, Runnable, Sequencer, SequencerAdapter};

pub struct Disruptor<E, Exe, S>
where
    E: Send + Sync + 'static,
    Exe: Executor,
    S: Sequencer,
{
    executor: Exe,
    ringbuffer: Arc<RingBuffer<E>>,
    repository: ConsumerRepository,
    sequencer: SequencerAdapter<S>,
}

impl<E, Exe, S> Disruptor<E, Exe, S>
where
    E: Send + Sync,
    Exe: Executor,
    S: Sequencer,
{
    pub fn new(
        executor: Exe,
        ringbuffer: Arc<RingBuffer<E>>,
        repository: ConsumerRepository,
        sequencer: SequencerAdapter<S>,
    ) -> Self {
        Disruptor {
            executor,
            ringbuffer,
            repository,
            sequencer,
        }
    }
}

impl<E, Exe, S> Disruptor<E, Exe, S>
where
    E: Send + Sync,
    Exe: Executor,
    S: Sequencer,
{
    pub fn start(&mut self) {
        for processor in self.repository.get_processors() {
            let processor = processor.clone();
            let r: Box<dyn Runnable> = Box::new(processor);
            self.executor.execute(r);
        }
    }

    pub fn stop(&mut self) {
        for processor in self.repository.get_mut_processors() {
            processor.halt();
        }
    }

    pub fn ringbuffer(&self) -> Arc<RingBuffer<E>> {
        self.ringbuffer.clone()
    }

    pub fn executor(&self) -> &dyn Executor {
        &self.executor
    }

    pub fn has_backlog(&mut self) -> bool {
        let cursor = self.sequencer.cursor().get();
        for processor in self.repository.get_processors() {
            if cursor > processor.sequence().get() {
                return true;
            }
        }
        false
    }
}

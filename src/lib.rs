pub mod adapters;
pub mod barrier;
pub mod batch_event_processor;
pub mod builder;
pub mod consumer_repository;
pub mod disruptor;
pub mod executor;
pub mod multi_producer;
pub mod publisher;
pub mod ringbuffer;
pub mod sequence;
pub mod shared_ref;
pub mod single_producer;
pub mod util;
pub mod wait_strategy;
mod work_processor;

pub use publisher::Publisher;
pub use shared_ref::SharedRef;
use std::pin::Pin;
use std::sync::Arc;

pub use adapters::{EventProcessorAdapter, SequencerAdapter};
pub use barrier::ProcessingSequenceBarrier;
pub use batch_event_processor::BatchEventProcessor;
pub use builder::DisruptorBuilder;
pub use consumer_repository::ConsumerRepository;
pub use disruptor::Disruptor;

pub use multi_producer::MultiProducer;
pub use multi_producer::MultiProducerSequencer;
pub use ringbuffer::RingBuffer;
pub use sequence::Sequence;
pub use single_producer::SingleProducer;
pub use single_producer::SingleProducerSequencer;

const STATE_IDLE: u8 = 0u8;
const STATE_HALTED: u8 = 1u8;
const STATE_RUNNING: u8 = 2u8;

pub trait Sequencer: Send + Sync + 'static {
    fn next(&mut self, sequence: i64) -> i64;
    fn publish(&self, sequence: i64);
    fn batch_publish(&self, low: i64, high: i64);
    fn highest_published(&self, next_sequence: i64, available_sequence: i64) -> i64;
    fn available(&self, sequence: i64) -> bool;
    fn cursor(&self) -> Arc<Sequence>;
    fn add_gating_sequence(&mut self, gating_sequence: Arc<Sequence>);
    fn buffer_size(&self) -> i64;
}

pub trait SequenceBarrier: Send + Sync {
    fn wait_for(&self, sequence: i64) -> Option<i64>;
    fn alert(&self);
    fn clear_alert(&self);
}

pub trait WaitStrategy: Send + Sync + 'static {
    fn wait_for<F: Fn() -> bool>(
        &self,
        sequence: i64,
        cursor: Arc<Sequence>,
        dependent_sequence: Vec<Arc<Sequence>>,
        check_alert: F,
    ) -> Option<i64>;

    fn signal_all_when_blocking(&self);
}

pub trait EventFactory<E> {
    fn new(&self) -> E;
}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn on_event(&mut self, event: &mut E, sequence: i64, end_of_batch: bool) {}
}

pub trait WorkHandler<E>: Send + Sync + 'static {
    fn on_event(&mut self, event: &mut E) {}
}

pub trait Runnable: Send + Sync {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

pub trait EventProcessor: Runnable {
    fn sequence(&self) -> Arc<Sequence>;
    fn running(self) -> bool;
    fn halt(&mut self);
}

pub trait Executor {
    fn execute(&self, runnable: Box<dyn Runnable>);
}

pub trait Producer<E, S>
where
    S: Sequencer,
{
    fn mut_sequencer(&mut self) -> &mut S;
}

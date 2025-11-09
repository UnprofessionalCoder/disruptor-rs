use std::sync::Arc;

use crate::{
    BatchEventProcessor, ConsumerRepository, Disruptor, EventFactory, EventHandler, EventProcessor,
    EventProcessorAdapter, Executor, MultiProducer, MultiProducerSequencer,
    ProcessingSequenceBarrier, Producer, RingBuffer, Sequence, Sequencer, SequencerAdapter,
    SingleProducer, SingleProducerSequencer, WaitStrategy,
};

pub struct DisruptorBuilder<E, Exe, W, S, P>
where
    E: Send + Sync + 'static,
    Exe: Executor + 'static,
    W: WaitStrategy + 'static,
    S: Sequencer,
    P: Producer<E, S>,
{
    executor: Exe,
    wait_strategy: Arc<W>,
    ringbuffer: Arc<RingBuffer<E>>,
    repository: ConsumerRepository,
    sequences: Vec<Arc<Sequence>>,
    sequence_barrier: Arc<ProcessingSequenceBarrier<W, S>>,
    producer: P,
    sequencer: SequencerAdapter<S>,
}

impl<E, Exe, W>
    DisruptorBuilder<E, Exe, W, SingleProducerSequencer, SingleProducer<E, SingleProducerSequencer>>
where
    E: Send + Sync + 'static,
    Exe: Executor + 'static,
    W: WaitStrategy + 'static,
{
    pub fn new_single_producer<F: EventFactory<E> + 'static>(
        buffer_size: i64,
        event_factory: F,
        executor: Exe,
        wait_strategy: W,
    ) -> Self {
        let sequencer = SingleProducerSequencer::new(buffer_size);
        let sequencer_adapter = SequencerAdapter::new(sequencer);
        let ringbuffer = Arc::new(RingBuffer::new(buffer_size as usize, event_factory));
        let producer = SingleProducer::new(sequencer_adapter.clone(), ringbuffer.clone());

        let cursor = sequencer_adapter.cursor();
        let wait_strategy_arc = Arc::new(wait_strategy);
        let sequence_barrier = Arc::new(ProcessingSequenceBarrier::new(
            wait_strategy_arc.clone(),
            sequencer_adapter.clone(),
            vec![cursor],
        ));

        Self {
            executor,
            wait_strategy: wait_strategy_arc,
            ringbuffer,
            repository: ConsumerRepository::new(),
            sequences: Vec::new(),
            sequence_barrier,
            producer,
            sequencer: sequencer_adapter,
        }
    }

    pub fn build_with_producer(
        self,
    ) -> (
        Disruptor<E, Exe, SingleProducerSequencer>,
        SingleProducer<E, SingleProducerSequencer>,
    ) {
        (
            Disruptor::new(
                self.executor,
                self.ringbuffer.clone(),
                self.repository,
                self.sequencer,
            ),
            self.producer,
        )
    }
}

impl<E, Exe, W>
    DisruptorBuilder<E, Exe, W, MultiProducerSequencer, MultiProducer<E, MultiProducerSequencer>>
where
    E: Send + Sync + 'static,
    Exe: Executor + 'static,
    W: WaitStrategy + 'static,
{
    pub fn new_multi_producer<F: EventFactory<E> + 'static>(
        buffer_size: i64,
        event_factory: F,
        executor: Exe,
        wait_strategy: W,
    ) -> DisruptorBuilder<E, Exe, W, MultiProducerSequencer, MultiProducer<E, MultiProducerSequencer>>
    {
        let sequencer = MultiProducerSequencer::new(buffer_size);
        let sequencer_adapter = SequencerAdapter::new(sequencer);
        let ringbuffer = Arc::new(RingBuffer::new(buffer_size as usize, event_factory));
        let producer = MultiProducer::new(sequencer_adapter.clone(), ringbuffer.clone());

        let cursor = sequencer_adapter.cursor();
        let wait_strategy_arc = Arc::new(wait_strategy);
        let sequence_barrier = Arc::new(ProcessingSequenceBarrier::new(
            wait_strategy_arc.clone(),
            sequencer_adapter.clone(),
            vec![cursor],
        ));

        DisruptorBuilder {
            executor,
            wait_strategy: wait_strategy_arc,
            ringbuffer,
            repository: ConsumerRepository::new(),
            sequences: Vec::new(),
            sequence_barrier,
            producer,
            sequencer: sequencer_adapter,
        }
    }

    pub fn build_with_producer(
        self,
    ) -> (
        Disruptor<E, Exe, MultiProducerSequencer>,
        MultiProducer<E, MultiProducerSequencer>,
    ) {
        (
            Disruptor::new(
                self.executor,
                self.ringbuffer,
                self.repository,
                self.sequencer,
            ),
            self.producer,
        )
    }

    pub fn producer(&self) -> MultiProducer<E, MultiProducerSequencer> {
        self.producer.clone()
    }
}

pub fn builder<E, F, Exe, W>(
    buffer_size: i64,
    event_factory: F,
    executor: Exe,
    wait_strategy: W,
) -> DisruptorBuilder<E, Exe, W, SingleProducerSequencer, SingleProducer<E, SingleProducerSequencer>>
where
    E: Send + Sync + 'static,
    F: EventFactory<E> + 'static,
    Exe: Executor + 'static,
    W: WaitStrategy + 'static,
{
    DisruptorBuilder::new_single_producer(buffer_size, event_factory, executor, wait_strategy)
}

impl<E, Exe, W, S, P> DisruptorBuilder<E, Exe, W, S, P>
where
    E: Send + Sync + 'static,
    Exe: Executor + 'static,
    W: WaitStrategy + 'static,
    S: Sequencer,
    P: Producer<E, S>,
{
    pub fn handler<H: EventHandler<E> + 'static>(mut self, handler: H) -> Self {
        let sequence_barrier = Arc::new(ProcessingSequenceBarrier::new(
            self.wait_strategy.clone(),
            self.sequencer.clone(),
            vec![self.sequencer.cursor()],
        ));

        let processor = BatchEventProcessor::new(
            handler,
            Arc::clone(&self.ringbuffer),
            Arc::clone(&sequence_barrier),
        );

        let sequence = processor.sequence();

        self.sequences.clear();

        self.sequences.push(Arc::clone(&sequence));
        self.repository
            .add(EventProcessorAdapter::new(Box::new(processor)));
        self.sequencer.add_gating_sequence(sequence);
        self.sequence_barrier = sequence_barrier;

        self
    }

    pub fn and<H: EventHandler<E> + 'static>(mut self, handler: H) -> Self {
        let processor = BatchEventProcessor::new(
            handler,
            Arc::clone(&self.ringbuffer),
            Arc::clone(&self.sequence_barrier),
        );

        let sequence = processor.sequence();
        self.sequences.push(Arc::clone(&sequence));
        self.repository
            .add(EventProcessorAdapter::new(Box::new(processor)));
        self.sequencer.add_gating_sequence(sequence);

        self
    }

    pub fn then<H: EventHandler<E> + 'static>(mut self, handler: H) -> Self {
        let wait_strategy = self.wait_strategy.clone();

        let sequence_barrier = Arc::new(ProcessingSequenceBarrier::new(
            wait_strategy,
            self.sequencer.clone(),
            self.sequences.clone(),
        ));

        let processor = BatchEventProcessor::new(
            handler,
            Arc::clone(&self.ringbuffer),
            Arc::clone(&sequence_barrier),
        );

        let sequence = processor.sequence();
        self.sequences.clear();
        self.sequences.push(Arc::clone(&sequence));
        self.repository
            .add(EventProcessorAdapter::new(Box::new(processor)));
        self.sequencer.add_gating_sequence(sequence);

        Self {
            sequence_barrier,
            ..self
        }
    }
}

// impl<E, Exe, W, S, P> DisruptorBuilder<E, Exe, W, S, P>
// where
//     E: Send + Sync + 'static,
//     Exe: Executor + 'static,
//     W: WaitStrategy + 'static,
//     S: Sequencer,
//     P: Producer<E, S>,
// {
//     pub fn producer(&self) -> P {
//         self.producer.clone()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EventFactory, EventHandler, executor::TokioExecutor, wait_strategy::BusySpinWaitStrategy,
    };

    struct TestEvent {
        value: String,
    }

    #[derive(Clone)]
    struct TestFactory;

    impl EventFactory<TestEvent> for TestFactory {
        fn new(&self) -> TestEvent {
            TestEvent {
                value: String::from("default"),
            }
        }
    }

    struct TestEventHandler {
        name: String,
    }

    impl EventHandler<TestEvent> for TestEventHandler {
        fn on_event(&mut self, event: &mut TestEvent, sequence: i64, _end_of_batch: bool) {
            println!(
                "{}: Event processed: {} at sequence {}",
                self.name, event.value, sequence
            );
        }
    }

    struct AnotherEventHandler {
        name: String,
    }

    impl EventHandler<TestEvent> for AnotherEventHandler {
        fn on_event(&mut self, event: &mut TestEvent, sequence: i64, _end_of_batch: bool) {
            println!(
                "{}: Processing event: {} at sequence {}",
                self.name, event.value, sequence
            );
        }
    }

    #[test]
    fn test_single_producer_builder() {
        let (mut disruptor, producer) = builder(
            8,
            TestFactory,
            TokioExecutor::new(1),
            BusySpinWaitStrategy {},
        )
        .handler(TestEventHandler {
            name: "handler1".to_string(),
        })
        .build_with_producer();

        assert!(!disruptor.has_backlog());
        let _ = producer;
    }

    #[test]
    fn test_multi_producer_builder() {
        let (mut disruptor, producer) = DisruptorBuilder::new_multi_producer(
            16,
            TestFactory,
            TokioExecutor::new(2),
            BusySpinWaitStrategy {},
        )
        .handler(TestEventHandler {
            name: "handler1".to_string(),
        })
        .build_with_producer();

        assert!(!disruptor.has_backlog());
        let _ = producer;
    }

    #[test]
    fn test_and_method() {
        let (mut disruptor, producer) = builder(
            8,
            TestFactory,
            TokioExecutor::new(1),
            BusySpinWaitStrategy {},
        )
        .handler(TestEventHandler {
            name: "handler1".to_string(),
        })
        .and(AnotherEventHandler {
            name: "handler2".to_string(),
        })
        .build_with_producer();

        assert!(!disruptor.has_backlog());
        let _ = producer;
    }

    #[test]
    fn test_then_method() {
        let (mut disruptor, producer) = builder(
            8,
            TestFactory,
            TokioExecutor::new(1),
            BusySpinWaitStrategy {},
        )
        .handler(TestEventHandler {
            name: "handler1".to_string(),
        })
        .then(AnotherEventHandler {
            name: "handler2".to_string(),
        })
        .build_with_producer();

        assert!(!disruptor.has_backlog());
        let _ = producer;
    }

    #[test]
    fn test_multiple_handlers() {
        let (mut disruptor, producer) = builder(
            8,
            TestFactory,
            TokioExecutor::new(1),
            BusySpinWaitStrategy {},
        )
        .handler(TestEventHandler {
            name: "handler1".to_string(),
        })
        .and(AnotherEventHandler {
            name: "handler2".to_string(),
        })
        .then(TestEventHandler {
            name: "handler3".to_string(),
        })
        .and(AnotherEventHandler {
            name: "handler4".to_string(),
        })
        .build_with_producer();

        assert!(!disruptor.has_backlog());
        let _ = producer;
    }
}

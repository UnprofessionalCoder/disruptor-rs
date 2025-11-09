//! Basic usage example for disruptor-rs
//!
//! This example demonstrates the fundamental usage pattern of the disruptor:
//! 1. Defining an event type
//! 2. Implementing EventFactory
//! 3. Implementing EventHandler  
//! 4. Creating and configuring the disruptor
//! 5. Publishing events
//! 6. Processing events

use disruptor_rs::{DisruptorBuilder, EventFactory, EventHandler, executor::TokioExecutor};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Define your event type
#[derive(Default)]
struct Event {
    value: String,
}

// Implement EventFactory for your event
struct EventFactoryImpl;

impl EventFactory<Event> for EventFactoryImpl {
    fn new(&self) -> Event {
        Event::default()
    }
}

// Implement EventHandler for processing events
struct EventHandlerImpl {
    name: String,
    counter: Arc<AtomicU64>,
}

impl EventHandler<Event> for EventHandlerImpl {
    fn on_event(&mut self, event: &mut Event, sequence: i64, end_of_batch: bool) {
        println!(
            "{} processed event: {} at sequence {}",
            self.name, event.value, sequence
        );
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

fn main() {
    let counter = Arc::new(AtomicU64::new(0));

    // Create a disruptor with multi-producer support
    let (mut disruptor, mut producer) = DisruptorBuilder::new_multi_producer(
        1024,
        EventFactoryImpl {},
        TokioExecutor::new(4),
        disruptor_rs::wait_strategy::BusySpinWaitStrategy {},
    )
    .handler(EventHandlerImpl {
        name: "handler1".to_string(),
        counter: counter.clone(),
    })
    .and(EventHandlerImpl {
        name: "handler2".to_string(),
        counter: counter.clone(),
    })
    .build_with_producer();

    disruptor.start();

    // Publish events
    for i in 0..1000 {
        producer.publish(|event| {
            event.value = format!("Event {}", i);
        });
    }

    // Wait for processing to complete
    while disruptor.has_backlog() {}

    disruptor.stop();
    println!("Processed {} events", counter.load(Ordering::Relaxed));
}

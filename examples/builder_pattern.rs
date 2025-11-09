//! Builder pattern examples for disruptor-rs
//!
//! This example demonstrates the fluent builder API with different configurations:
//! 1. Multi producer with parallel processing
//! 2. Multi producer with sequential processing

use disruptor_rs::{DisruptorBuilder, EventFactory, EventHandler, executor::TokioExecutor};

// Example event type
#[derive(Default)]
struct MyEvent {
    data: String,
}

// Example factory
struct MyFactory;

impl EventFactory<MyEvent> for MyFactory {
    fn new(&self) -> MyEvent {
        MyEvent::default()
    }
}

// Example handler
struct MyHandler;

impl EventHandler<MyEvent> for MyHandler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        println!(
            "MyHandler processing: {} at sequence {}",
            event.data, sequence
        );
    }
}

// Another example handler
struct AnotherHandler;

impl EventHandler<MyEvent> for AnotherHandler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        println!(
            "AnotherHandler processing: {} at sequence {}",
            event.data, sequence
        );
    }
}

// Sequential handler
struct NextHandler;

impl EventHandler<MyEvent> for NextHandler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        println!(
            "NextHandler processing: {} at sequence {}",
            event.data, sequence
        );
    }
}

fn main() {
    // Multi producer example with parallel processing
    let (mut disruptor1, producer1) = DisruptorBuilder::new_multi_producer(
        1024,
        MyFactory,
        TokioExecutor::new(4),
        disruptor_rs::wait_strategy::BusySpinWaitStrategy {},
    )
    .handler(MyHandler)
    .and(AnotherHandler) // Parallel processing
    .build_with_producer();

    // Multi producer example with sequential processing
    let (mut disruptor2, producer2) = DisruptorBuilder::new_multi_producer(
        1024,
        MyFactory,
        TokioExecutor::new(4),
        disruptor_rs::wait_strategy::BusySpinWaitStrategy {},
    )
    .handler(MyHandler)
    .then(NextHandler) // Sequential processing
    .build_with_producer();

    println!("Builder examples created successfully");
    println!("Example 1: Parallel processing with MyHandler and AnotherHandler");
    println!("Example 2: Sequential processing with MyHandler then NextHandler");
}

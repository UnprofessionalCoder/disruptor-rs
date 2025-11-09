//! Chained processing example for disruptor-rs
//!
//! This example demonstrates complex handler chaining with both
//! parallel and sequential processing stages.

use disruptor_rs::{DisruptorBuilder, EventFactory, EventHandler, executor::TokioExecutor};

// Example event type
#[derive(Default)]
struct MyEvent {
    id: u64,
    processed_by: Vec<String>,
}

// Example factory
struct MyFactory;

impl EventFactory<MyEvent> for MyFactory {
    fn new(&self) -> MyEvent {
        MyEvent::default()
    }
}

// Stage 1 handler - parallel processing
struct Stage1Handler;

impl EventHandler<MyEvent> for Stage1Handler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        event.processed_by.push("Stage1".to_string());
        println!(
            "Stage1 processed event {} at sequence {}",
            event.id, sequence
        );
    }
}

// Stage 2 handler - parallel with Stage1
struct Stage2Handler;

impl EventHandler<MyEvent> for Stage2Handler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        event.processed_by.push("Stage2".to_string());
        println!(
            "Stage2 processed event {} at sequence {}",
            event.id, sequence
        );
    }
}

// Stage 3 handler - sequential after Stage1+2
struct Stage3Handler;

impl EventHandler<MyEvent> for Stage3Handler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        event.processed_by.push("Stage3".to_string());
        println!(
            "Stage3 processed event {} at sequence {} (after Stage1+2)",
            event.id, sequence
        );
    }
}

// Stage 4 handler - parallel with Stage3
struct Stage4Handler;

impl EventHandler<MyEvent> for Stage4Handler {
    fn on_event(&mut self, event: &mut MyEvent, sequence: i64, end_of_batch: bool) {
        event.processed_by.push("Stage4".to_string());
        println!(
            "Stage4 processed event {} at sequence {} (parallel with Stage3)",
            event.id, sequence
        );
    }
}

fn main() {
    let (mut disruptor, producer) = DisruptorBuilder::new_multi_producer(
        1024,
        MyFactory,
        TokioExecutor::new(4),
        disruptor_rs::wait_strategy::BusySpinWaitStrategy {},
    )
    .handler(Stage1Handler) // Parallel
    .and(Stage2Handler) // Parallel
    .then(Stage3Handler) // Sequential after Stage1+2
    .and(Stage4Handler) // Parallel with Stage3
    .build_with_producer();

    println!("Chained processing example created successfully");
    println!("Processing flow:");
    println!("1. Stage1Handler (parallel)");
    println!("2. Stage2Handler (parallel with Stage1)");
    println!("3. Stage3Handler (sequential after Stage1+2)");
    println!("4. Stage4Handler (parallel with Stage3)");
    println!();
    println!("Key points:");
    println!("- .handler() starts a new parallel chain");
    println!("- .and() adds parallel processing within the same chain");
    println!("- .then() adds sequential processing after the previous chain");
}

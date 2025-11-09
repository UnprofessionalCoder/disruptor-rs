//! Multi producer mode examples for disruptor-rs
//!
//! This example demonstrates different multi-producer configurations:
//! 1. Multi producer with BusySpin wait strategy
//! 2. Multi producer with Yielding wait strategy

use disruptor_rs::{DisruptorBuilder, EventFactory, EventHandler, executor::TokioExecutor};

// Example event type
#[derive(Default)]
struct MyEvent {
    value: u64,
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
        println!("Processing event {} at sequence {}", event.value, sequence);
    }
}

fn main() {
    // Multi producer with BusySpin wait strategy
    let (mut disruptor1, producer1) = DisruptorBuilder::new_multi_producer(
        2048,
        MyFactory,
        TokioExecutor::new(4),
        disruptor_rs::wait_strategy::BusySpinWaitStrategy {},
    )
    .handler(MyHandler)
    .build_with_producer();

    // Multi producer with Yielding wait strategy
    let (mut disruptor2, producer2) = DisruptorBuilder::new_multi_producer(
        1024,
        MyFactory,
        TokioExecutor::new(2),
        disruptor_rs::wait_strategy::YieldingWaitStrategy {},
    )
    .handler(MyHandler)
    .build_with_producer();

    println!("Multi producer examples created successfully");
    println!("Example 1: 2048 buffer size, BusySpin wait strategy, 4 worker threads");
    println!("Example 2: 1024 buffer size, Yielding wait strategy, 2 worker threads");
    println!();
    println!("BusySpinWaitStrategy:");
    println!("- Use case: Lowest latency, high CPU usage");
    println!("- Best for: Real-time systems where latency is critical");
    println!();
    println!("YieldingWaitStrategy:");
    println!("- Use case: Balanced latency and CPU usage");
    println!("- Best for: General purpose applications");
}

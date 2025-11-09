//! Multiple parallel handler chains example for disruptor-rs
//!
//! This example demonstrates creating two completely independent parallel
//! handler chains that process events concurrently.

use disruptor_rs::{
    DisruptorBuilder, EventFactory, EventHandler, executor::TokioExecutor,
    wait_strategy::BusySpinWaitStrategy,
};

#[derive(Default, Clone)]
struct Event {
    id: String,
}

struct EventFactoryImpl;

impl EventFactory<Event> for EventFactoryImpl {
    fn new(&self) -> Event {
        Event::default()
    }
}

// First chain handlers
struct HandlerA {
    name: String,
}

impl EventHandler<Event> for HandlerA {
    fn on_event(&mut self, event: &mut Event, sequence: i64, end_of_batch: bool) {
        println!("{} processing event {}", self.name, event.id);
    }
}

struct HandlerB {
    name: String,
}

impl EventHandler<Event> for HandlerB {
    fn on_event(&mut self, event: &mut Event, sequence: i64, end_of_batch: bool) {
        println!("{} processing after HandlerA: {}", self.name, event.id);
    }
}

// Second chain handlers
struct HandlerC {
    name: String,
}

impl EventHandler<Event> for HandlerC {
    fn on_event(&mut self, event: &mut Event, sequence: i64, end_of_batch: bool) {
        println!("{} processing event {}", self.name, event.id);
    }
}

struct HandlerD {
    name: String,
}

impl EventHandler<Event> for HandlerD {
    fn on_event(&mut self, event: &mut Event, sequence: i64, end_of_batch: bool) {
        println!("{} processing after HandlerC: {}", self.name, event.id);
    }
}

fn main() {
    let (mut disruptor, mut producer) = DisruptorBuilder::new_multi_producer(
        1024,
        EventFactoryImpl {},
        TokioExecutor::new(4),
        BusySpinWaitStrategy {},
    )
    // First chain: HandlerA → HandlerB (sequential)
    .handler(HandlerA {
        name: "Chain1-HandlerA".to_string(),
    })
    .then(HandlerB {
        name: "Chain1-HandlerB".to_string(),
    })
    // Second chain: HandlerC → HandlerD (sequential), parallel with first chain
    .handler(HandlerC {
        name: "Chain2-HandlerC".to_string(),
    })
    .then(HandlerD {
        name: "Chain2-HandlerD".to_string(),
    })
    .build_with_producer();

    disruptor.start();

    for i in 0..10 {
        producer.publish(|event| {
            event.id = format!("EVENT-{:03}", i);
        });
    }

    disruptor.stop();

    println!();
    println!("Processing Flow:");
    println!("Event Publication");
    println!("    │");
    println!("    ├── HandlerA → HandlerB (Chain 1)");
    println!("    │");
    println!("    └── HandlerC → HandlerD (Chain 2)");
    println!();
    println!("Key Characteristics:");
    println!("- HandlerA and HandlerC: Execute in parallel (separate .handler() calls)");
    println!("- HandlerB: Executes after HandlerA completes (.then())");
    println!("- HandlerD: Executes after HandlerC completes (.then())");
    println!("- Each chain has independent sequence barriers for true parallelism");
}

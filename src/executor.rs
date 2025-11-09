use async_executor::Executor;
use tokio::runtime::{self, Runtime};

use crate::Runnable;

pub struct TokioExecutor {
    rt: Runtime,
}

impl TokioExecutor {
    pub fn new(worker_thread_num: usize) -> Self {
        let rt = runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(worker_thread_num)
            .build()
            .unwrap();
        TokioExecutor { rt }
    }
}

impl crate::Executor for TokioExecutor {
    fn execute(&self, mut runnable: Box<dyn Runnable>) {
        self.rt.spawn(async move {
            runnable.run().await;
        });
    }
}

pub struct SmolExecutor<'a> {
    executor: Executor<'a>,
}

impl<'a> SmolExecutor<'a> {
    pub fn new() -> Self {
        let executor = Executor::new();
        SmolExecutor { executor }
    }
}

impl<'a> crate::Executor for SmolExecutor<'a> {
    fn execute(&self, mut runnable: Box<dyn Runnable>) {
        self.executor
            .spawn(async move {
                runnable.run().await;
            })
            .detach();
    }
}

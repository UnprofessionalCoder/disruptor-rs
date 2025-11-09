use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use crate::{EventProcessor, Runnable, Sequencer, SharedRef};

pub struct EventProcessorAdapter {
    processor: SharedRef<Box<dyn EventProcessor>>,
}

impl EventProcessorAdapter {
    pub fn new(processor: Box<dyn EventProcessor>) -> Self {
        EventProcessorAdapter {
            processor: SharedRef::new(processor),
        }
    }
}

impl Runnable for EventProcessorAdapter {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.processor.run()
    }
}

impl Clone for EventProcessorAdapter {
    fn clone(&self) -> Self {
        Self {
            processor: self.processor.clone(),
        }
    }
}

impl Deref for EventProcessorAdapter {
    type Target = SharedRef<Box<dyn EventProcessor>>;

    fn deref(&self) -> &Self::Target {
        &self.processor
    }
}

impl DerefMut for EventProcessorAdapter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.processor
    }
}

pub struct SequencerAdapter<S>
where
    S: Sequencer,
{
    sequencer: SharedRef<S>,
}

impl<S> SequencerAdapter<S>
where
    S: Sequencer,
{
    pub fn new(sequencer: S) -> Self {
        SequencerAdapter {
            sequencer: SharedRef::new(sequencer),
        }
    }
}

impl<S> Clone for SequencerAdapter<S>
where
    S: Sequencer,
{
    fn clone(&self) -> Self {
        Self {
            sequencer: self.sequencer.clone(),
        }
    }
}

impl<S> Deref for SequencerAdapter<S>
where
    S: Sequencer,
{
    type Target = SharedRef<S>;

    fn deref(&self) -> &Self::Target {
        &self.sequencer
    }
}

impl<S> DerefMut for SequencerAdapter<S>
where
    S: Sequencer,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sequencer
    }
}

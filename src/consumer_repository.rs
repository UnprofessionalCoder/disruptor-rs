use crate::EventProcessorAdapter;

pub struct ConsumerRepository {
    processors: Vec<EventProcessorAdapter>,
}

impl ConsumerRepository {
    pub fn new() -> ConsumerRepository {
        ConsumerRepository { processors: vec![] }
    }

    pub fn add(&mut self, processor: EventProcessorAdapter) {
        self.processors.push(processor);
    }

    pub fn size(&self) -> usize {
        self.processors.len()
    }

    pub fn get_processors(&self) -> &Vec<EventProcessorAdapter> {
        &self.processors
    }

    pub fn get_mut_processors(&mut self) -> &mut Vec<EventProcessorAdapter> {
        &mut self.processors
    }
}

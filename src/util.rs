use std::sync::Arc;

use crate::Sequence;

pub(crate) fn minimum_sequence(sequences: &Vec<Arc<Sequence>>) -> i64 {
    sequences.iter().map(|s| s.get()).min().unwrap_or(0)
}

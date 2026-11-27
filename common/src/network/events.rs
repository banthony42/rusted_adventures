use crate::{network::SequenceNumber, CellCoord, Position};

#[derive(Clone)]
pub enum EventKind {
    RequestPath,
}

impl EventKind {
    const VARIANTS: [Self; 1] = [Self::RequestPath];
}
pub enum EntityEvent {
    RequestPath(Position, CellCoord),
}

#[derive(Default)]
pub struct EntityEventSequences {
    request_path: SequenceNumber,
}

impl EntityEventSequences {
    pub fn next(&mut self, kind: EventKind) -> u32 {
        match kind {
            EventKind::RequestPath => self.request_path.increment(),
        }
    }

    pub fn get(&self, kind: EventKind) -> u32 {
        match kind {
            EventKind::RequestPath => self.request_path.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EntityEventSequences, EventKind};

    fn some_function_that_increment_sequence(sequence: &mut EntityEventSequences, kind: EventKind) {
        sequence.next(kind);
    }

    fn some_function_that_read_sequence(sequence: &EntityEventSequences, kind: EventKind) -> u32 {
        sequence.get(kind)
    }

    #[test]
    fn entity_event_sequence_are_persistent() {
        let mut sequence = EntityEventSequences::default();

        for kind in EventKind::VARIANTS {
            some_function_that_increment_sequence(&mut sequence, kind.clone());
            some_function_that_increment_sequence(&mut sequence, kind.clone());

            assert_eq!(some_function_that_read_sequence(&sequence, kind), 2);
        }
    }
}

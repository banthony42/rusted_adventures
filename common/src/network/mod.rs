use rand::{
    distr::{Distribution, StandardUniform},
    RngExt,
};

use crate::grpc_codegen::EntityDespawn;
use crate::grpc_codegen::{
    server_entity_event::Event::{Despawn, Move, Spawn},
    EntityMove, EntitySpawn,
};
use crate::grpc_codegen::{Entity as RpcEntity, Location as RpcLocation, ServerEntityEvent};

pub mod conversion;
pub mod events;

pub trait ServerEntityEventExtension {
    fn movement(uuid: String, location: RpcLocation) -> Self;
    fn spawn(entity: RpcEntity) -> Self;
    fn despawn(uuid: String) -> Self;
}

impl ServerEntityEventExtension for ServerEntityEvent {
    fn movement(uuid: String, location: RpcLocation) -> Self {
        ServerEntityEvent {
            event: Some(Move(EntityMove {
                uuid,
                new_location: Some(location),
            })),
        }
    }

    fn spawn(entity: RpcEntity) -> Self {
        ServerEntityEvent {
            event: Some(Spawn(EntitySpawn {
                new_entity: Some(entity),
            })),
        }
    }

    fn despawn(uuid: String) -> Self {
        ServerEntityEvent {
            event: Some(Despawn(EntityDespawn { uuid })),
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct SequenceNumber(u32);

impl SequenceNumber {
    /// Increment the inner value by one, and return the result.
    ///
    /// If an overflow occure, the value is reset to 0.
    pub fn increment(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }
}

impl Distribution<SequenceNumber> for StandardUniform {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> SequenceNumber {
        SequenceNumber(rng.random())
    }
}

#[cfg(test)]
mod tests {
    use super::SequenceNumber;

    fn some_function_that_increment_sequence(sequence: &mut SequenceNumber) {
        sequence.increment();
    }

    fn some_function_that_read_sequence(sequence: &SequenceNumber) -> u32 {
        sequence.0
    }

    #[test]
    fn sequence_number_is_persistent() {
        let mut sequence = SequenceNumber::default();

        some_function_that_increment_sequence(&mut sequence);
        some_function_that_increment_sequence(&mut sequence);

        assert_eq!(some_function_that_read_sequence(&sequence), 2);
    }

    #[test]
    fn sequence_random_number_is_persistent() {
        let mut sequence = rand::random::<SequenceNumber>();
        let initial_value = sequence.0;

        some_function_that_increment_sequence(&mut sequence);
        some_function_that_increment_sequence(&mut sequence);

        assert_eq!(
            some_function_that_read_sequence(&sequence),
            initial_value + 2
        );
    }
}

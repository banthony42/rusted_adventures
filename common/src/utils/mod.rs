use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

#[derive(Clone, Copy)]
pub struct SequenceNumber(u32);

impl SequenceNumber {
    /// Init inner value with a new random sequence number.
    pub fn new() -> Self {
        Self {
            0: rand::random::<u32>(),
        }
    }

    /// Increment the inner value by one, and return the result.
    ///
    /// If an overflow occure, the value is reset to 0.
    pub fn increment(&mut self) -> u32 {
        self.0 = match self.0.checked_add(1) {
            Some(result) => result,
            None => 0,
        };
        self.0
    }
}

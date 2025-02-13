use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> u128 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis();
}

pub struct WorldEngine {
    ups: u128,
}

impl WorldEngine {
    pub fn new() -> Self {
        WorldEngine { ups: 1000 }
    }

    pub async fn run(&self) {
        let mut ts = get_timestamp();
        loop {
            if get_timestamp() - ts > self.ups {
                // println!("[World server]: update ...");
                ts = get_timestamp();
            }
        }
    }
}

use std::collections::HashMap;

use crate::utils::get_timestamp;

#[derive(Clone)]
struct HashData<R>
where
    R: Clone,
{
    expire_at: u128,
    data: R,
}

pub struct Record<R>
where
    R: Clone,
{
    hash: HashMap<String, HashData<R>>,
    // complete: Vec<String>,
}

impl<R> Record<R>
where
    R: Clone,
{
    /// Return `true` if the inner `hdata` TTL is expired.
    fn is_expired(hdata: &HashData<R>) -> bool {
        hdata.expire_at != 0 && get_timestamp() > hdata.expire_at
    }

    fn ttl_into_expire_time(ttl: u32) -> u128 {
        if ttl == 0 {
            return 0;
        }
        get_timestamp() + ttl as u128 * 1000
    }

    /// Create redis proxy to mimic key value storage.
    /// Goal here is just to use few basic redis feature, avoiding redis dependency.
    ///
    /// This code mimic redis interface, therefore if the need grow in the future
    /// we can easily replace it by a real redis instance and library (server side)
    /// or it will be totally reworked / enhanced (client side).
    ///
    /// For all the reason mention above i have follow the KISS principle for the implementation.
    pub fn new() -> Self {
        Self {
            hash: HashMap::new(),
        }
    }

    /// Mimic `DEL` redis command.
    ///
    /// Remove the given `key`.
    pub fn del(&mut self, key: &String) {
        self.hash.remove(key);
    }

    /// Mimic `EXPIRE` redis command.
    ///
    /// Set a timeout on `key` in seconds. After the timeout has expired, the key will be deleted at the next `hget` or `update` call.
    pub fn expire(&mut self, key: &String, ttl: u32) {
        if let Some(hdata) = self.hash.get_mut(key) {
            hdata.expire_at = Self::ttl_into_expire_time(ttl)
        }
    }

    /// Mimic `SET` redis command.
    /// Set `key`to hold the `value: R`.
    ///
    /// If `key` already holds a value, it is overwritten.
    /// `ttl` : Set the specified expire time, in seconds.
    ///
    /// Any previous `TTL` associated with the key is discarded on successful `SET` operation.
    pub fn set(&mut self, key: String, value: R, ttl: Option<u32>) {
        self.hash.insert(
            key,
            HashData {
                expire_at: Self::ttl_into_expire_time(ttl.unwrap_or(0)),
                data: value,
            },
        );
    }

    pub fn get(&mut self, key: &String) -> Option<R> {
        if let Some(hdata) = self.hash.get(key) {
            if Self::is_expired(&hdata) {
                self.del(key);
                return None;
            }
            return Some(hdata.data.clone());
        }
        None
    }

    /// Mimic the automatic `key` removal of redis according to their `TTL`.
    ///
    /// Browse all the data, and delete them if their respective `TTL` has expired.
    pub fn update(&mut self) {
        self.hash
            .retain(|_, hdata| Self::is_expired(hdata) == false);
    }

    // For test

    pub fn db_len(&self) -> usize {
        self.hash.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread::sleep, time::Duration};

    #[derive(Clone, PartialEq, Debug)]
    struct CMsg {
        seq_number: i32,
    }

    #[test]
    fn store_persistent_key_value() {
        let key = "mykey1".to_owned();
        let data = CMsg { seq_number: 42 };

        let mut myrec = Record::<CMsg>::new();
        myrec.set(key.clone(), data.clone(), None);

        assert_eq!(myrec.db_len(), 1);

        myrec.set("mykey2".to_owned(), data.clone(), None);
        assert_eq!(myrec.db_len(), 2);
        sleep(Duration::from_millis(200));
        assert_eq!(myrec.get(&key), Some(data));

        // `get` should only return the value it's not a `pop` operation.
        assert_eq!(myrec.db_len(), 2);
    }

    #[test]
    fn delete_key() {
        let key = "mykey1".to_owned();
        let data = CMsg { seq_number: 42 };

        let mut myrec = Record::<CMsg>::new();
        myrec.set(key.clone(), data.clone(), None);
        assert_eq!(myrec.db_len(), 1);

        myrec.del(&key);
        assert_eq!(myrec.db_len(), 0);
        assert_eq!(myrec.get(&key), None);
    }

    #[test]
    fn expired_key_with_get() {
        let key1 = "mykey1".to_owned();
        let key2 = "mykey2".to_owned();
        let data1 = CMsg { seq_number: 42 };
        let data2 = CMsg { seq_number: 21 };

        let mut myrec = Record::<CMsg>::new();
        myrec.set(key1.clone(), data1.clone(), Some(3));
        myrec.set(key2.clone(), data2.clone(), None);

        // Test key1 still exist after 1 second
        sleep(Duration::from_millis(1000));
        assert_eq!(myrec.get(&key1), Some(data1));

        // Test key1 is expired but key2 still exist
        sleep(Duration::from_millis(2200));

        // db_len still equal to 2 because expired key removal are performed at `get` or `update` call
        assert_eq!(myrec.db_len(), 2);

        assert_eq!(myrec.get(&key1), None);
        assert_eq!(myrec.get(&key2), Some(data2));
        assert_eq!(myrec.db_len(), 1);

        // Set ttl to key2 and ensure it's deleted when timeout
        myrec.expire(&key2, 1);
        sleep(Duration::from_millis(1200));
        assert_eq!(myrec.get(&key2), None);
        assert_eq!(myrec.db_len(), 0);
    }

    #[test]
    fn expired_key_with_update() {
        let key1 = "mykey1".to_owned();
        let key2 = "mykey2".to_owned();
        let key3 = "mykey3".to_owned();
        let data = CMsg { seq_number: 42 };

        let mut myrec = Record::<CMsg>::new();
        myrec.set(key1, data.clone(), Some(1));
        myrec.set(key2, data.clone(), Some(3));
        myrec.set(key3, data, None);

        sleep(Duration::from_millis(100));
        myrec.update();
        assert_eq!(myrec.db_len(), 3);

        sleep(Duration::from_millis(1000));
        myrec.update();
        assert_eq!(myrec.db_len(), 2);

        sleep(Duration::from_millis(2000));
        myrec.update();
        assert_eq!(myrec.db_len(), 1);
    }

    #[test]
    fn key_overwrite() {
        let key = "mykey1".to_owned();
        let data = CMsg { seq_number: 42 };
        let data2 = CMsg { seq_number: 21 };

        let mut myrec = Record::<CMsg>::new();
        myrec.set(key.clone(), data, None);
        myrec.set(key.clone(), data2.clone(), None);

        assert_eq!(myrec.get(&key), Some(data2));
    }

    #[test]
    fn several_recorder() {
        let key1 = "mykey1".to_owned();
        let key2 = "mykey2".to_owned();
        let key3 = "mykey3".to_owned();
        let data1 = CMsg { seq_number: 42 };

        let mut struct_recorder = Record::<CMsg>::new();
        let mut int_recorder = Record::<i32>::new();
        let mut bool_recorder = Record::new();

        struct_recorder.set(key1.clone(), data1.clone(), None);
        int_recorder.set(key2.clone(), 42, None);
        bool_recorder.set(key3.clone(), false, None);

        sleep(Duration::from_millis(100));

        assert_eq!(struct_recorder.get(&key1), Some(data1));
        assert_eq!(int_recorder.get(&key2), Some(42));
        assert_eq!(bool_recorder.get(&key3), Some(false));
    }
}

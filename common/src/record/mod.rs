use std::collections::HashMap;

use crate::utils::get_timestamp;

const NO_TTL: u128 = 0;

#[derive(Clone)]
struct HashData<R>
where
    R: Clone,
{
    expire_at: u128,
    data: Vec<(String, R)>,
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
        hdata.expire_at != NO_TTL && get_timestamp() > hdata.expire_at
    }

    /// Create redis proxy to mimic hash storage.
    /// Goal here is just to use few basic redis feature, avoiding redis dependency.
    ///
    /// This code mimic redis interface, therefore if the need grow in the future
    /// we can easily replace this by a real redis instance and library (server side)
    /// or it will be totally rework (client side).
    ///
    /// For all the reason mention above i have follow the KISS principle for the implementation.
    pub fn new() -> Self {
        Self {
            hash: HashMap::new(),
        }
    }

    /// Set `field`:`value` pair in the HashMap stored at `key`.
    /// Mimic `HSET` redis command.
    pub fn hset(&mut self, key: String, field: String, value: R) {
        self.hash
            .entry(key)
            .and_modify(|hdata| hdata.data.push((field.clone(), value.clone())))
            .or_insert(HashData {
                expire_at: NO_TTL,
                data: vec![(field, value)],
            });
    }

    /// Set a timeout on `key` in seconds. After the timeout has expired, the key will be deleted at the next `hget` or `update` call.
    /// Mimic `EXPIRE` redis command.
    pub fn expire(&mut self, key: &String, ttl: u32) {
        if let Some(hdata) = self.hash.get_mut(key) {
            hdata.expire_at = get_timestamp() + (ttl * 1000) as u128;
        }
    }

    /// Return all `fields` and `values` of the hash stored at `key`.
    /// Mimic `HGETALL` redis command.
    /// Limitation: `key` should exist. Wildcard pattern like redis does, such as: `HGETALL MYKEY*` is not implemented.
    pub fn hgetall(&mut self, key: &String) -> Option<Vec<(String, R)>> {
        if let Some(hdata) = self.hash.get(key) {
            if Self::is_expired(&hdata) {
                self.del(key);
                return None;
            }
            return Some(hdata.data.clone());
        }
        None
    }

    /// Remove the given `key`.
    /// Mimic `DEL` redis command.
    pub fn del(&mut self, key: &String) {
        self.hash.remove(key);
    }

    /// Return all data that match the `predicate`.
    /// The returned data are also removed from the hash.
    /// In redis, this could be impl. with a redis Lua script.
    pub fn hgetall_match<F>(&mut self, mut predicate: F) -> Vec<Vec<(String, R)>>
    where
        F: FnMut(&Vec<(String, R)>) -> bool,
    {
        let found: Vec<Vec<(String, R)>> = self
            .hash
            .iter_mut()
            .filter(|entry| predicate(&entry.1.data))
            .map(|(_, v)| {
                // Tricks to consume data avoiding additional hash browse:
                // Mark each data expired, to take advantage of the next
                // the next `update` call.
                v.expire_at = get_timestamp() + 1;
                v.data.clone()
            })
            .collect();
        found
    }

    /// Browse all the data, and delete them if their respective `TTL` has expired.
    /// Mimic the automatic `key` removal of redis according to their `TTL`.
    pub fn update(&mut self) {
        self.hash
            .retain(|_, hdata| Self::is_expired(hdata) == false);
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
    fn hset_hgetall_one_data() {
        let mut myrec = Record::<CMsg>::new();

        let key = "mykey1".to_owned();
        let field = "request".to_owned();
        myrec.hset(key.clone(), field.clone(), CMsg { seq_number: 42 });
        let result = myrec.hgetall(&key);
        assert_eq!(
            result,
            Some(vec![("request".to_owned(), CMsg { seq_number: 42 })])
        );
    }

    #[test]
    fn hset_hgetall_with_several_data() {
        let mut myrec = Record::<CMsg>::new();

        let key1 = "mykey1".to_owned();
        let key2 = "mykey2".to_owned();
        let req_field = "request".to_owned();
        let res_field = "response".to_owned();

        myrec.hset(key1.clone(), req_field.clone(), CMsg { seq_number: 42 });
        myrec.hset(key2.clone(), req_field.clone(), CMsg { seq_number: 21 });
        myrec.hset(key1.clone(), res_field.clone(), CMsg { seq_number: 42 });

        let result1 = myrec.hgetall(&key1);
        assert_eq!(
            result1,
            Some(vec![
                ("request".to_owned(), CMsg { seq_number: 42 }),
                ("response".to_owned(), CMsg { seq_number: 42 })
            ])
        );

        let result2 = myrec.hgetall(&key2);
        assert_eq!(
            result2,
            Some(vec![("request".to_owned(), CMsg { seq_number: 21 })])
        );
    }

    #[test]
    fn delete_key() {
        let mut myrec = Record::<CMsg>::new();

        let key = "mykey1".to_owned();
        let field = "request".to_owned();
        myrec.hset(key.clone(), field.clone(), CMsg { seq_number: 42 });

        myrec.del(&key);
        let result = myrec.hgetall(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn expired_key() {
        let mut myrec = Record::<CMsg>::new();

        let key = "mykey1".to_owned();
        let field = "request".to_owned();

        myrec.hset(key.clone(), field.clone(), CMsg { seq_number: 42 });

        let ttl: u32 = 2;
        myrec.expire(&key, ttl);

        sleep(Duration::from_secs((ttl + 1) as u64));
        let result = myrec.hgetall(&key);
        assert_eq!(result, None);
    }

    #[test]
    fn key_still_exist() {
        let mut myrec = Record::<CMsg>::new();

        let key = "mykey1".to_owned();
        let field = "request".to_owned();

        myrec.hset(key.clone(), field.clone(), CMsg { seq_number: 42 });
        myrec.expire(&key, 5);

        sleep(Duration::from_secs(1));

        let result = myrec.hgetall(&key);
        assert_eq!(
            result,
            Some(vec![("request".to_owned(), CMsg { seq_number: 42 })])
        );
    }

    #[test]
    fn update_data() {
        let mut myrec = Record::<CMsg>::new();

        let key1 = "mykey1".to_owned();
        let key2 = "mykey2".to_owned();
        let key3 = "mykey3".to_owned();
        let req_field = "request".to_owned();

        myrec.hset(key1.clone(), req_field.clone(), CMsg { seq_number: 84 });
        myrec.hset(key2.clone(), req_field.clone(), CMsg { seq_number: 42 });
        myrec.hset(key3.clone(), req_field.clone(), CMsg { seq_number: 21 });

        myrec.expire(&key1, 5);
        myrec.expire(&key2, 3);
        myrec.expire(&key3, 1);

        sleep(Duration::from_millis(1500));
        myrec.update();

        assert_eq!(
            myrec.hgetall(&key1),
            Some(vec![("request".to_owned(), CMsg { seq_number: 84 })])
        );
        assert_eq!(
            myrec.hgetall(&key2),
            Some(vec![("request".to_owned(), CMsg { seq_number: 42 })])
        );
        assert_eq!(myrec.hgetall(&key3), None);

        sleep(Duration::from_secs(4));
        myrec.update();

        assert_eq!(myrec.hgetall(&key1), None);
        assert_eq!(myrec.hgetall(&key2), None);
        assert_eq!(myrec.hgetall(&key3), None);
    }
}

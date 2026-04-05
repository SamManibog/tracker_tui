pub struct TinyMap<K: Ord, V>(Vec<(K, V)>);

impl<K: Ord, V> TinyMap<K, V> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// gets the value at the given key
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let insert_index = self.0.binary_search_by_key(&key, |pair| &pair.0);
        match insert_index {
            Ok(index) => Some(&self.0[index].1),
            Err(_) => None,
        }
    }
}

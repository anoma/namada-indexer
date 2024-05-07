use bimap::BiMap;
use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Checksums(BiMap<String, String>);

impl Checksums {
    pub fn init(&mut self) {
        let mut clean_checksums = BiMap::default();
        for (key, value) in self.0.clone() {
            let key = key.strip_suffix(".wasm").unwrap().to_owned();
            let value = value.split('.').nth(1).unwrap().to_owned();
            clean_checksums.insert(key, value);
        }
        self.0 = clean_checksums;
    }

    pub fn get_name_by_id(&self, hash: &str) -> Option<String> {
        self.0.get_by_right(hash).map(|data| data.to_owned())
    }

    pub fn get_id_by_name(&self, name: &str) -> Option<String> {
        self.0.get_by_left(name).map(|data| data.to_owned())
    }
}

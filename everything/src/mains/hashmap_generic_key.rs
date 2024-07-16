use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct GroupTestStorage {
    storage: HashMap<u64, HashMap<i32, Aggregator>>,
}
//
// impl Storage for GroupTestStorage {
//     fn add_element(&mut self, ad: &PeeledAd) {
//         self.storage
//             .entry(ad.group_id)
//             .or_insert(HashMap::new())
//             .entry(ad.ab_test_id)
//             .or_insert(Aggregator::default())
//             .add(ad);
//     }
//
//     fn to_output(&self) -> OutputAggregateData {
//         self.into()
//     }
// }
//
// impl From<&GroupTestStorage> for OutputAggregateData {
//     fn from(value: &GroupTestStorage) -> Self {
//         let mut output = OutputAggregateData {
//             meta_data: MetaData {
//                 key_desc: vec![GROUP_ID_KEY.to_string(), AB_TEST_ID_KEY.to_string()],
//                 key_type: vec!["u64".to_string(), "i32".to_string()],
//             },
//             data: Vec::new(),
//         };
//
//         for (key_1, value_1) in &value.storage {
//             for (key_2, value_2) in value_1 {
//                 let mut keys: Vec<u8> = Vec::new();
//                 keys.extend_from_slice(&key_1.to_be_bytes());
//                 keys.extend_from_slice(&key_2.to_be_bytes());
//                 output.data.push(Data {
//                     keys,
//                     value: value_2.into(),
//                 })
//             }
//         }
//
//         output
//     }
// }

pub struct GenericStorage<T: Key, F: Fn(&PeeledAd) -> T> {
    data: HashMap<T, Aggregator>,
    key_desc: Vec<String>,
    key_extractor: F,
}

impl<T: Key, F: Fn(&PeeledAd) -> T> GenericStorage<T, F> {
    pub fn new(key_desc: Vec<String>, key_extractor: F) -> Self {
        Self {
            data: HashMap::new(),
            key_desc,
            key_extractor,
        }
    }
}

impl<T: Key, F: Fn(&PeeledAd) -> T> Debug for GenericStorage<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericStorage")
            .field("data", &self.data)
            .field("key_desc", &self.key_desc)
            .finish()
    }
}

impl<T: Key, F: Fn(&PeeledAd) -> T + Send> Storage for GenericStorage<T, F> {
    fn add_element(&mut self, ad: &PeeledAd) {
        let key = (self.key_extractor)(ad);
        self.data
            .entry(key)
            .or_insert(Aggregator::default())
            .add(&ad);
    }

    fn to_output(&self) -> OutputAggregateData {
        OutputAggregateData {
            meta_data: MetaData {
                key_desc: self.key_desc.clone(),
                key_type: T::description(),
            },
            data: self
                .data
                .iter()
                .map(|(key, agg)| Data {
                    keys: key.to_bytes(),
                    value: agg.into(),
                })
                .collect(),
        }
    }
}

pub trait Key: Send + Debug + PartialEq + Hash + Eq {
    fn to_bytes(&self) -> Vec<u8>;
    fn description() -> Vec<String>;
}

fn main() {
    let mut storage =
        GenericStorage::new(vec!["group_id".to_owned(), "test_id".to_owned()], |ad| {
            (ad.group_id, ad.ab_test_id)
        });

    storage.add_element(&PeeledAd::default());

    let output = storage.to_output();
    println!("OUTPUT: {:#?}", output);
}

pub trait Storage: Debug + Send {
    fn add_element(&mut self, ad: &PeeledAd);
    fn to_output(&self) -> OutputAggregateData;
}

//  DO NOT CHANGE THIS ##################333

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PeeledAd {
    pub group_id: u64,
    pub feed_tag: i32,
    pub ab_test_id: i32,
    pub zone_id: i32,
    pub clicked: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputAggregateData {
    pub meta_data: MetaData,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetaData {
    pub key_desc: Vec<String>,
    pub key_type: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Data {
    pub keys: Vec<u8>,
    pub value: AggregatorOutput,
}

#[derive(Clone, Debug, Default, Copy, PartialEq)]
pub struct AggregatorOutput {
    pub impressions: u64,
    pub clicks: u64,
}

#[derive(Debug, Default)]
pub struct Aggregator {
    pub impressions: AtomicU64,
    pub clicks: AtomicU64,
}

impl Aggregator {
    pub fn add(&mut self, ad: &PeeledAd) {
        self.impressions.fetch_add(1, Ordering::Relaxed);
        if ad.clicked {
            self.clicks.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl From<&Aggregator> for AggregatorOutput {
    fn from(value: &Aggregator) -> Self {
        Self {
            impressions: value.impressions.load(Ordering::Acquire),
            clicks: value.clicks.load(Ordering::Acquire),
        }
    }
}

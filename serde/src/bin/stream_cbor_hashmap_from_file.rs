use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let path = "test-file.cbor";

    {
        let file = open_file(path)?;

        let mut writer = BufWriter::new(file);
        let mut cbor_bytes = vec![];

        for i in iter_serde_data() {
            cbor_bytes.clear();
            ciborium::into_writer(&test_data(), &mut cbor_bytes)?;
            writer.write_all(&cbor_bytes)?;
        }

        writer.flush()?;
    }

    // {
    //     let file = File::open(path)?;
    //     let reader = BufReader::new(file);
    //
    //     ciborium::de::from_reader()
    //
    //     // let res: Aggregator = ciborium::from_reader(reader)?;
    //     // println!("{:#?}", res);
    // }

    Ok(())
}

fn open_file(path: impl AsRef<Path>) -> std::io::Result<File> {
    if path.as_ref().exists() {
        fs::remove_file(path.as_ref())?;
    }

    let parent = path.as_ref().parent();

    if let Some(parent) = parent {
        fs::create_dir_all(parent)?;
    }

    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .append(false)
        .open(path)
}

// DATA

fn iter_serde_data() -> impl IntoIterator<Item = SerdeAggregate> {
    test_data()
        .aggregate
        .into_iter()
        .map(|(k, v)| SerdeAggregate { key: k, data: v })
}

fn test_data() -> Aggregator {
    let mut agg = Aggregator::default();

    for i in 0..5u8 {
        let mut data = AggregateData::default();
        data.page_url = format!("page-url-{}", i);

        for i in 0..3 {
            let mut groups = HashMap::new();
            for i in 0..10 {
                groups.insert(i, Stats::default());
            }
            data.per_feedtag_stats.insert(i, groups);
        }

        agg.aggregate.insert(UrlKey::new([i; 16]), data);
    }

    agg
}

// just store it as a standalone objects, then we cna stream it 1 by 1 instead of loading
// whole hashmap of size few GBs into a memory
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SerdeAggregate {
    key: UrlKey,
    data: AggregateData,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Aggregator {
    aggregate: HashMap<UrlKey, AggregateData>, // Key is MD5 hash of page url
}

// this cant be used as a key in json map, has to be a string
#[derive(Clone, Debug, PartialOrd, Serialize, Deserialize)]
pub struct UrlKey {
    pub md5: [u8; 16], // url hash
    pub hash: u64,     // hashed hash :D used for hashmap to skip calculation every time
}

impl Display for UrlKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = self
            .md5
            .to_vec()
            .into_iter()
            .map(|n| n.to_string())
            .collect::<Vec<String>>()
            .join("-");
        write!(f, "{}", str)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AggregateData {
    pub page_url: String,
    pub per_feedtag_stats: HashMap<i32, HashMap<u32, Stats>>, // <Feedtag, <GroupId, Stats>>
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub impressions: u32,
    pub clicks: u32,
    pub visible_clicks: u32,
    pub visible_impressions: u32,
}

impl Hash for UrlKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq<UrlKey> for UrlKey {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for UrlKey {}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

impl UrlKey {
    pub fn new(md5: [u8; 16]) -> Self {
        Self {
            hash: calculate_hash(&md5),
            md5,
        }
    }
}

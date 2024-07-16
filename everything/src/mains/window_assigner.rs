use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::iter::Iterator;

// Assuming the structs are already defined as provided
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OutputAggregate {
    pub from_ts: u64,
    pub to_ts: u64,
    pub aggregates: Vec<OutputAggregateData>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OutputAggregateData {
    pub meta_data: MetaData,
    pub data: Vec<Data>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetaData {
    pub key_desc: Vec<String>,
    pub key_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Data {
    pub keys: Vec<u8>,
    pub value: AggregatorOutput,
}

#[derive(Clone, Debug, Default, Copy, Serialize, PartialEq)]
pub struct AggregatorOutput {
    pub impressions: u64,
    pub clicks: u64,
}

fn split(output: OutputAggregate, max_part_size: usize) -> Vec<OutputAggregate> {
    let mut parts = Vec::new();
    let mut current_part_size = 0;

    let mut current_output = OutputAggregate {
        from_ts: output.from_ts,
        to_ts: output.to_ts,
        aggregates: vec![],
    };

    for aggregate in output.aggregates {
        if aggregate.data.is_empty() {
            continue;
        }

        let mut current_aggregate = OutputAggregateData {
            meta_data: aggregate.meta_data.clone(),
            data: vec![],
        };
        let mut current_data = Vec::new();

        // all entries have the same size -> no need to serialize every single one
        let entry_size = serde_json::to_string(aggregate.data.get(0).unwrap())
            .unwrap()
            .len(); // safe unwrap

        for entry in aggregate.data {
            if current_part_size + entry_size > max_part_size && !current_data.is_empty() {
                // finish part and create a new one
                let mut current_c = current_output.clone();
                let mut current_aggregate_c = current_aggregate.clone();
                current_aggregate_c.data.append(&mut current_data);

                current_c.aggregates.push(current_aggregate_c);
                parts.push(current_c);

                current_output.aggregates.clear();
                current_data.clear();
                current_part_size = 0;
            }

            current_data.push(entry);
            current_part_size += entry_size;
        }

        // add aggregate
        current_aggregate.data.append(&mut current_data);
        current_output.aggregates.push(current_aggregate);
    }

    if !current_output.aggregates.is_empty() {
        parts.push(current_output);
    }

    parts
}

fn main() {
    // Example usage
    let mut aggregates = Vec::with_capacity(4);

    for i in 0..3 {
        let meta_data = MetaData {
            key_desc: vec![format!("{i}-desc")],
            key_type: vec![format!("{i}-type")],
        };

        let mut data = Vec::with_capacity(5);
        for i in 0..5 {
            data.push(Data {
                keys: vec![i],
                value: AggregatorOutput {
                    impressions: i as u64,
                    clicks: i as u64,
                },
            });
        }

        aggregates.push(OutputAggregateData { meta_data, data });
    }

    let output = OutputAggregate {
        from_ts: 0,
        to_ts: 10,
        aggregates,
    };

    let json = serde_json::to_string(&output).unwrap();
    println!(
        "Serialized size: {} Kb",
        json.as_bytes().len() as f64 / 1024.
    );

    let parts = split(output, 128);

    for (i, part) in parts.iter().enumerate() {
        println!(
            "\n\nPart {}, Size: {}Kb, PART: \n{:#?}",
            i + 1,
            serde_json::to_string(&part).unwrap().as_bytes().len() as f64 / 1024.,
            part
        );
    }
}

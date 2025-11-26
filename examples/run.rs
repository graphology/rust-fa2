use rand::prelude::*;
use std::collections::HashMap;
use std::fs::File;

use clap::Parser;

use fa2::{FA2Data, FA2Layout, FA2Settings};

#[derive(Parser, Debug)]
struct Args {
    /// Path to target CSV file
    path: String,

    /// Index of source column
    source: usize,

    /// Index of target column
    target: usize,
}

impl Args {
    fn delimiter(&self) -> u8 {
        if self.path.ends_with(".tsv") {
            b'\t'
        } else {
            b','
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file = File::open(&args.path)?;

    let mut reader = simd_csv::ReaderBuilder::new()
        .delimiter(args.delimiter())
        .from_reader(file);

    let mut record = simd_csv::ByteRecord::new();

    let mut rng = rand::rng();

    let mut node_index: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut layout_data = FA2Data::<f32>::new();

    while reader.read_byte_record(&mut record)? {
        let source = &record[args.source];
        let target = &record[args.target];

        let i = *node_index
            .entry(source.to_vec())
            .or_insert_with(|| layout_data.add_node(rng.random(), rng.random()));

        let j = *node_index
            .entry(target.to_vec())
            .or_insert_with(|| layout_data.add_node(rng.random(), rng.random()));

        layout_data.add_edge(i, j);
    }

    let settings = FA2Settings::<f32>::from_graph_order(layout_data.order());
    let mut layout = FA2Layout::with_settings(settings, layout_data);

    layout.run(1);

    let mut writer = simd_csv::Writer::from_writer(std::io::stdout());
    writer.write_record_no_quoting(["x", "y"])?;

    for (x, y) in layout.positions() {
        writer.write_record_no_quoting([x.to_string(), y.to_string()])?;
    }

    writer.flush()?;

    Ok(())
}

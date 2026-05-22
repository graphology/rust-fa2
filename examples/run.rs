use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::time::SystemTime;

use btoi::btoi;
use clap::Parser;
use fa2::{FA2Data, FA2Settings};
use rand::prelude::*;

#[derive(Parser, Debug)]
struct Args {
    /// Path to target CSV file
    path: String,

    /// Index of source column
    source: usize,

    /// Index of target column
    target: usize,

    /// Number of iterations to run
    #[arg(long, default_value = "10")]
    iterations: usize,

    /// Dump layout state every n iteration
    #[arg(long)]
    dump_every: Option<usize>,

    /// Just apply a circular layout and exit
    #[arg(long)]
    circular: bool,

    /// Use Barnes-Hut?
    #[arg(long)]
    barnes_hut: bool,

    /// Verbose
    #[arg(short, long)]
    verbose: bool,

    /// Parallel
    #[arg(short, long)]
    parallel: bool,

    /// Number of threads to use
    #[arg(short, long)]
    threads: Option<usize>,

    #[arg(long)]
    range: Option<usize>,
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

    if let Some(threads) = args.threads {
        if args.verbose {
            eprintln!("using {} threads", threads);
        }

        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .unwrap();
    }

    let file = File::open(&args.path)?;

    let mut reader = simd_csv::ReaderBuilder::new()
        .delimiter(args.delimiter())
        .from_reader(file);

    let mut record = simd_csv::ByteRecord::new();

    let mut rng = rand::rng();

    let mut node_index: HashMap<Vec<u8>, usize> = HashMap::new();

    let mut layout_data = FA2Data::<f32>::new();

    if let Some(n) = args.range {
        for _ in 0..n + 1 {
            layout_data.add_node(rng.random(), rng.random());
        }
    }

    while reader.read_byte_record(&mut record)? {
        let source = &record[args.source];
        let target = &record[args.target];

        let (i, j) = if args.range.is_some() {
            (btoi(source)?, btoi(target)?)
        } else {
            (
                *node_index
                    .entry(source.to_vec())
                    .or_insert_with(|| layout_data.add_node(rng.random(), rng.random())),
                *node_index
                    .entry(target.to_vec())
                    .or_insert_with(|| layout_data.add_node(rng.random(), rng.random())),
            )
        };

        layout_data.add_edge(i, j);
    }

    if args.verbose {
        eprintln!(
            "data loaded: {} nodes, {} edges!",
            layout_data.order(),
            layout_data.size()
        );
    }

    let mut settings =
        FA2Settings::<f32>::from_graph_order(layout_data.order()).parallel(args.parallel);

    settings = if args.barnes_hut {
        settings.with_barnes_hut()
    } else {
        settings.with_pairwise_repulsion()
    };

    if args.verbose {
        eprintln!("{:?}", settings);
    }

    if args.dump_every.is_some() {
        fs::remove_dir_all("dump")?;
        fs::create_dir_all("dump")?;
    }

    if args.circular {
        layout_data.apply_circular_layout();
    } else {
        let mut layout = settings.build(layout_data);

        for i in 0..args.iterations {
            let now = if args.verbose {
                Some(SystemTime::now())
            } else {
                None
            };

            let movement = layout.epoch();

            if args.verbose {
                eprintln!(
                    "Epoch n°{}, movement={}, time={:?}",
                    i + 1,
                    movement,
                    now.unwrap().elapsed().unwrap()
                );
            }

            if let Some(every) = args.dump_every {
                if i % every == 0 {
                    dump(
                        File::create(&format!("dump/{:>05}.csv", i))?,
                        args.range.is_some(),
                        &node_index,
                        layout.data(),
                    )?;
                }
            }
        }

        layout_data = layout.into_data();
    }

    fn dump<W: Write>(
        w: W,
        nameless: bool,
        node_index: &HashMap<Vec<u8>, usize>,
        layout_data: &FA2Data<f32>,
    ) -> anyhow::Result<()> {
        let mut writer = simd_csv::Writer::from_writer(w);

        if nameless {
            writer.write_record_no_quoting(["x", "y"])?;
        } else {
            writer.write_record_no_quoting(["node", "x", "y"])?;
        }

        let reverse_node_index = node_index
            .into_iter()
            .map(|(k, v)| (v, k))
            .collect::<HashMap<_, _>>();

        for (i, (x, y)) in layout_data.positions().enumerate() {
            if nameless {
                writer.write_record([x.to_string().as_bytes(), y.to_string().as_bytes()])?;
            } else {
                writer.write_record([
                    reverse_node_index.get(&i).unwrap(),
                    x.to_string().as_bytes(),
                    y.to_string().as_bytes(),
                ])?;
            }
        }

        Ok(writer.flush()?)
    }

    dump(
        std::io::stdout(),
        args.range.is_some(),
        &node_index,
        &layout_data,
    )?;

    Ok(())
}

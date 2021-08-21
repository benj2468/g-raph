use g_raph::{
    self,
    graph::{
        edge::Edge, static_a::coloring::Color, streaming::coloring::StreamColoring,
        GraphWithRecaller, Graphed,
    },
    printdur,
    random_graph::{bernoulli::BernoulliGraphDistribution, uniform::UniformGraphDistribution},
    start_dur,
};
use itertools::Itertools;
use rand::prelude::Distribution;
use std::{
    f32::INFINITY,
    fs::File,
    io::{self, BufRead},
};

macro_rules! graph_test {
    ($n:expr, $edges:expr) => {{
        println!("-------------- Starting Graph Test --------------");

        let start = start_dur!();
        let base = StreamColoring::init($n as u32, 1, 0.01);
        let mut next_colorers: Vec<_> = (1..($n.log2().floor() as u32))
            // let mut next_colorers: Vec<_> = vec![]
            .into_iter()
            .filter_map(|i| {
                let k = 2_u32.pow(i) as u64;
                base.new_k($n as u32, k)
            })
            .collect();

        let mut colorers = vec![base];
        colorers.append(&mut next_colorers);

        let mut whole_graph = GraphWithRecaller::new(Default::default());

        printdur!("Initialization", start);
        println!("--------------------------------------------------");
        let start = start_dur!();

        let mut len = 0;
        for (edge, c) in $edges {
            for colorer in &mut colorers {
                colorer.feed(edge, c)
            }
            whole_graph.add_edge(edge);
            len += 1;
        }

        println!("Stream Length: {}", len);
        printdur!("Stream", start);
        println!("--------------------------------------------------");

        let mut min_color = INFINITY as usize;
        for (i, colorer) in colorers.into_iter().enumerate() {
            if let Some(coloring) = colorer.query() {
                let count = coloring.values().unique().count();
                println!("Estimate #{} -> {} Coloring", i, count);
                if count < min_color {
                    min_color = count;
                }
            } else {
                println!("Estimate #{} -> Not Sparse Enough", i);
            }
        }

        let actual = whole_graph.color_degeneracy().values().unique().count();

        println!("--------------------------------------------------");
        println!("Results: (K + 1): {:?}, Streaming: {:?}", actual, min_color);
        println!("-------------- Completed Graph Test --------------");

        (actual, min_color)
    }};
}

macro_rules! graph_file_test {
    ($file_name:expr, $n:expr, $split:expr) => {{
        let file = File::open(format!("./big_graphs/{}", $file_name)).unwrap();

        let edges = io::BufReader::new(file)
            .lines()
            .filter_map(|r| r.ok())
            .map(|line| {
                let mut split = line.split($split);
                let v1: u32 = split.next().unwrap().parse().unwrap();
                let v2: u32 = split.next().unwrap().parse().unwrap();

                (Edge::<u32, ()>::init(v1, v2), true)
            });

        graph_test!($n, edges)
    }};
}

#[test]
#[ignore]
fn facebook_combined() {
    let (actual, stream_res) = graph_file_test!("facebook_combined.txt", 4_039_f32, " ");

    assert!(actual <= stream_res)
}

#[test]
#[ignore]
fn facebook_artists() {
    let (actual, stream_res) = graph_file_test!("artist_edges.txt", 50_515_f32, ",");

    assert!(actual <= stream_res)
}

#[test]
#[ignore]
fn youtube() {
    let (actual, stream_res) = graph_file_test!("com-youtube.ungraph.txt", 1_134_890_f32, "\t");

    assert!(actual <= stream_res)
}

#[test]
#[ignore]
fn ratbrain() {
    let (actual, stream_res) = graph_file_test!("ratbrain.txt", 496_f32, " ");

    assert!((actual as isize - stream_res as isize).abs() <= 1 || actual <= stream_res)
}

#[test]
#[ignore]
fn fake_test() {
    let (actual, stream_res) = graph_file_test!("fake.txt", 10_f32, " ");

    assert!(actual <= stream_res)
}

#[test]
#[ignore]
fn erdos_renyi_sample_dense() {
    let n = 1500;

    let (actual, stream_res) =
        graph_test!(n as f32, BernoulliGraphDistribution::init(n, 0.9).unwrap());

    assert!(actual <= stream_res);
}
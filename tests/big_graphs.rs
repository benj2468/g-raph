use g_raph::{
    self,
    graph::{
        edge::Edge, static_a::coloring::Colorer,
        streaming::coloring::ack::StreamColoring as ACKColorer,
        streaming::coloring::bcg::StreamColoring, Graph, GraphWithRecaller, Graphed,
    },
    printdur,
    random_graph::bernoulli::BernoulliGraphDistribution,
    start_dur,
};
use itertools::Itertools;

use rand::prelude::Distribution;
use std::{
    collections::HashSet,
    convert::TryInto,
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
                    break;
                }
            } else {
                println!("Estimate #{} -> Not Sparse Enough", i);
            }
        }

        let actual = whole_graph.color_degeneracy().values().unique().count();

        println!("--------------------------------------------------");
        println!("Results: (K + 1): {:?}, Streaming: {:?}", actual, min_color);
        println!("-------------- Completed Graph Test --------------");

        assert!((actual as isize - min_color as isize).abs() <= 2 || actual <= min_color);

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

fn ack_test_graph(graph: Graph<u32, ()>) {
    let max_degree: u32 = graph
        .adj_list()
        .values()
        .map(|n| n.len())
        .max()
        .unwrap()
        .try_into()
        .unwrap();

    let mut ack_colorer = ACKColorer::init(graph.vertices().into_iter().collect(), max_degree);

    println!("Initialization: {:?}", ack_colorer);

    // This should not need to be cloned
    for edge in graph.clone() {
        ack_colorer.feed((edge, true))
    }

    println!("Stream Completed");

    let coloring = ack_colorer.query().unwrap();

    println!("Colors Used: {:?}", coloring.values().unique().count());

    assert!(graph.is_proper(coloring));
}

fn ack_test(file_name: &str, vertices: u32, separator: &str) {
    let file = File::open(format!("./big_graphs/{}", file_name)).unwrap();

    let mut graph = Graph::default();

    io::BufReader::new(file)
        .lines()
        .filter_map(|r| r.ok())
        .map(|line| {
            let mut split = line.split(separator);
            let v1: u32 = split.next().unwrap().parse().unwrap();
            let v2: u32 = split.next().unwrap().parse().unwrap();

            (Edge::<u32, ()>::init(v1, v2), true)
        })
        .for_each(|(edge, _)| graph.add_edge(edge));

    ack_test_graph(graph);
}

#[test]
#[ignore]
fn facebook_combined() {
    graph_file_test!("facebook_combined.txt", 4_039_f32, " ");
}

#[test]
#[ignore]
fn facebook_combined_ack() {
    ack_test("facebook_combined.txt", 4_039, " ");
}

#[test]
#[ignore]
fn facebook_artists() {
    graph_file_test!("artist_edges.txt", 50_515_f32, ",");
}

#[test]
#[ignore]
fn youtube() {
    graph_file_test!("com-youtube.ungraph.txt", 1_134_890_f32, "\t");
}

#[test]
#[ignore]
fn ratbrain() {
    graph_file_test!("ratbrain.txt", 496_f32, " ");
}

#[test]
#[ignore]
fn ratbrain_ack() {
    ack_test("ratbrain.txt", 496, " ");
}

#[test]
#[ignore]
fn fake_test() {
    graph_file_test!("fake.txt", 10_f32, " ");
}

#[test]
#[ignore]
fn erdos_renyi_sample_dense() {
    let n = 1500;
    graph_test!(n as f32, BernoulliGraphDistribution::init(n, 0.9).unwrap());
}

#[test]
#[ignore]
fn erdos_renyi_sample_dense_ack() {
    let n = 300;
    let graph = BernoulliGraphDistribution::init(n, 0.9).unwrap();
    let mut rng = rand::thread_rng();

    ack_test_graph(graph.sample(&mut rng));
}

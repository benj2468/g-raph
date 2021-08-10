use g_raph::{
    self,
    graph::{
        edge::Edge, static_a::coloring::Color, streaming::coloring::StreamColoring,
        GraphWithRecaller, Graphed,
    },
    random_graph::uniform::UniformGraphDistribution,
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
        let mut colorers: Vec<_> = (0..($n.log2().floor() as u32))
            .into_iter()
            .map(|i| {
                let k = 2_u32.pow(i) as u64;
                StreamColoring::init($n as u32, k, 0.01)
            })
            .collect();

        let mut whole_graph = GraphWithRecaller::new(Default::default());

        for edge in $edges {
            for colorer in &mut colorers {
                colorer.feed(edge, true)
            }
            whole_graph.add_edge(edge);
        }

        let mut min_color = INFINITY as usize;
        for colorer in colorers.into_iter() {
            if let Some(coloring) = colorer.query() {
                let count = coloring.values().unique().count();
                if count < min_color {
                    min_color = count;
                }
            }
        }

        let actual = whole_graph.color_degeneracy().values().unique().count();

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

                Edge::<u32, ()>::init(v1, v2)
            });

        graph_test!($n, edges)
    }};
}

#[test]
#[ignore]
fn facebook_combined() {
    let res = graph_file_test!("facebook_combined.txt", 4_039_f32, " ");

    println!("{:?}", res);
}

#[test]
#[ignore]
fn facebook_artists() {
    let res = graph_file_test!("artist_edges.txt", 50_515_f32, ",");

    println!("{:?}", res);
}

#[test]
#[ignore]
fn youtube() {
    let res = graph_file_test!("com-youtube.ungraph.txt", 1_134_890_f32, "\t");

    println!("{:?}", res);
}

#[test]
fn ratbrain() {
    let res = graph_file_test!("ratbrain.txt", 496_f32, " ");

    println!("{:?}", res)
}

#[test]
fn fake_test() {
    let res = graph_file_test!("fake.txt", 10_f32, " ");

    println!("{:?}", res)
}

#[test]
fn sampled_graph() {
    let n = 500;
    let distribution = UniformGraphDistribution::init(n, 200_000);
    let mut rng = rand::thread_rng();
    let graph_edges: Vec<_> = distribution.sample(&mut rng);

    let (actual, min_color) = graph_test!(n as f32, graph_edges);

    assert!(actual <= min_color);
}

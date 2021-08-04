use g_raph::{
    self,
    graph::{
        edge::Edge, static_a::coloring::Color, streaming::coloring::StreamColoring,
        GraphWithRecaller, Graphed,
    },
};
use itertools::Itertools;
use std::{
    f32::INFINITY,
    fs::File,
    io::{self, BufRead},
};

macro_rules! big_graph_test {
    ($file_name:expr, $n:expr, $split:expr) => {{
        let file = File::open(format!("./big_graphs/{}", $file_name)).unwrap();

        // I think the first case doesn't really work. Not sure yet
        let mut colorers: Vec<_> = (0..($n.log2().floor() as u32))
            // let mut colorers: Vec<_> = (0..1)
            .into_iter()
            .map(|i| {
                let k = (2 as u32).pow(i) as u64;
                StreamColoring::init($n as u32, k, 0.01)
            })
            .collect();

        println!("Completed Initialization");

        let mut whole_graph = GraphWithRecaller::new(Default::default());

        for line in io::BufReader::new(file).lines() {
            if let Ok(line) = line {
                let mut split = line.split($split);
                let v1: u32 = split.next().unwrap().parse().unwrap();
                let v2: u32 = split.next().unwrap().parse().unwrap();

                let edge = Edge::<u32, ()>::init(v1, v2);

                for colorer in &mut colorers {
                    colorer.feed(edge, true)
                }
                whole_graph.add_edge(edge);
            }
        }

        println!("Completed Stream");

        let mut min_color = INFINITY as usize;
        for colorer in colorers.into_iter() {
            if let Some(coloring) = colorer.query() {
                let count = coloring.values().unique().count();
                println!("{:?}", count);
                if count < min_color {
                    min_color = count
                }
            } else {
                println!("bad batch?")
            }
        }

        let actual = whole_graph.color_degeneracy().values().unique().count();

        (min_color, actual)
    }};
}

#[test]
#[ignore]
fn facebook_combined() {
    let res = big_graph_test!("facebook_combined.txt", 4_039_f32, " ");

    println!("{:?}", res);
}

#[test]
#[ignore]
fn facebook_artists() {
    let res = big_graph_test!("artist_edges.txt", 50_515_f32, ",");

    println!("{:?}", res);
}

#[test]
#[ignore]
fn youtube() {
    let res = big_graph_test!("com-youtube.ungraph.txt", 1_134_890_f32, "\t");

    println!("{:?}", res);
}

#[test]
fn fake_test() {
    let res = big_graph_test!("fake.txt", 10_f32, " ");

    println!("{:?}", res)
}

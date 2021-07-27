use g_raph::{
    self,
    graph::{edge::Edge, streaming::coloring::StreamColoring},
};
use itertools::Itertools;
use std::{
    cmp::min,
    f32::INFINITY,
    fs::File,
    io::{self, BufRead},
};

macro_rules! big_graph_test {
    ($file_name:expr, $n:expr) => {{
        let file = File::open(format!("./big_graphs/{}", $file_name)).unwrap();

        let mut colorers: Vec<_> = (0..($n.log2().floor() as u32))
            .into_iter()
            .map(|i| {
                let k = (2 as u32).pow(i) as u64;
                StreamColoring::init($n as u32, k)
            })
            .collect();

        for line in io::BufReader::new(file).lines() {
            if let Ok(line) = line {
                let mut split = line.split(" ");
                let v1: u32 = split.next().unwrap().parse().unwrap();
                let v2: u32 = split.next().unwrap().parse().unwrap();

                let edge = Edge::<u32, ()>::init(v1, v2);

                for colorer in &mut colorers {
                    colorer.feed(edge, true)
                }
            }
        }

        let mut min_color = INFINITY as usize;
        for colorer in colorers {
            if let Some(coloring) = colorer.query() {
                let count = coloring.iter().unique().count();

                min_color = min(min_color, count);
            }
        }

        min_color
    }};
}

#[test]
fn stanford_graphs() {
    let min_color = big_graph_test!("facebook_combined.txt", 4039_f32);

    println!("{:?}", min_color)
}

#[test]
fn fake_test() {
    let min_color = big_graph_test!("fake.txt", 10_f32);

    println!("{:?}", min_color)
}

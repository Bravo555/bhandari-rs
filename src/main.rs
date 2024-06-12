use clap::{arg, command, Parser};
use pathfinding::prelude::*;
use std::{collections::HashMap, fs};

use anyhow::Context;

#[derive(Debug, Clone, Parser)]
#[command()]
struct Args {
    #[arg()]
    file: String,

    #[arg()]
    start: u32,

    #[arg()]
    to: u32,

    #[arg()]
    k: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let edges: anyhow::Result<Vec<_>> = fs::read_to_string(args.file)?
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .map(parse_edge)
        .collect();
    let edges = edges?;

    let result = bhandari(&edges, args.start, args.to, args.k).context("getting disjoint paths")?;

    println!("{result:?}");

    Ok(())
}

fn bhandari(_graph: &[Edge], start: u32, end: u32, k: usize) -> anyhow::Result<Vec<Vec<u32>>> {
    // dijkstra calls a function at each step to get list of next nodes it goes to, so transform our
    // edge list to lambda that returns `to` nodes for a given node
    let shortest_path = {
        let successors = |current_node: &u32| {
            _graph
                .iter()
                .filter(|edge| edge.from == *current_node)
                .map(|&Edge { to, weight, .. }| (to, weight))
                .collect::<Vec<_>>()
        };

        // find shortest path P_1 from s to t
        let (shortest_path, _cost) =
            dijkstra(&start, successors, |current_node| *current_node == end)
                .context("this graph doesn't contain such path")?;

        shortest_path
    };

    let mut paths: Vec<Vec<u32>> = Vec::with_capacity(k);
    paths.push(shortest_path);

    for _ in 0..(k - 1) {
        // if node-disjoint path split the intermediate nodes of all Px where x < i
        // we use link-disjoint, so skip

        // Replace each link of all P_x where x < i with a reverse link of inverted link weight in the original graph
        let mut graph: HashMap<(u32, u32), i32> = HashMap::from_iter(
            _graph
                .iter()
                .map(|edge| ((edge.from, edge.to), edge.weight)),
        );

        for path in &paths {
            for link in path.windows(2) {
                let from = link[0];
                let to = link[1];

                let (_, weight) = graph
                    .remove_entry(&(from, to))
                    .expect("link should be present");
                graph.insert((to, from), -weight);
            }
        }

        // Find the shortest path Pi from node s to node t
        let successors = |current_node: &u32| {
            graph
                .iter()
                .filter(|((from, _), _)| *current_node == *from)
                .map(|((_, to), weight)| (*to, *weight))
                .collect::<Vec<_>>()
        };
        let (shortest_path, _cost) =
            dijkstra(&start, successors, |current_node| *current_node == end)
                .context("this graph doesn't contain such path")?;

        paths.push(shortest_path);

        // Remove all overlapping links to get i disjoint paths P_x where x ≤ i
        let mut unique_links = paths[0]
            .windows(2)
            .map(|link| (link[0], link[1]))
            .collect::<Vec<_>>();

        for path in paths[1..].iter() {
            let links = path
                .windows(2)
                .map(|link| (link[0], link[1]))
                .collect::<Vec<_>>();

            for (from, to) in links {
                if let Some(pos) = unique_links
                    .iter()
                    .position(|(f, t)| (from == *f && to == *t) || (from == *t) && (to == *f))
                {
                    unique_links.remove(pos);
                } else {
                    unique_links.push((from, to));
                }
            }
        }

        let starting_links = unique_links
            .iter()
            .filter(|(from, _)| *from == start)
            .copied()
            .collect::<Vec<_>>();

        paths = starting_links
            .iter()
            .map(|(start, starting_next)| {
                let mut current_node = *starting_next;
                let mut path = vec![*start, current_node];

                while current_node != end {
                    let pos = unique_links
                        .iter()
                        .position(|(from, _)| current_node == *from)
                        .expect("should exist");
                    let (_, next) = unique_links.remove(pos);

                    path.push(next);
                    current_node = next;
                }
                path
            })
            .collect::<Vec<_>>();
    }

    Ok(paths)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Edge {
    from: u32,
    to: u32,
    weight: i32,
}

fn parse_edge(line: &str) -> anyhow::Result<Edge> {
    let mut parts = line.split_whitespace();

    let from = parts.next().context("no starting node")?.parse()?;

    let weight = parts.next().context("no weight")?.parse()?;

    let to = parts.next().context("no finish node")?.parse()?;

    Ok(Edge { from, to, weight })
}

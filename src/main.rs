use pathfinding::prelude::*;
use std::fs;

use anyhow::{ensure, Context};

fn main() -> anyhow::Result<()> {
    let edges: anyhow::Result<Vec<_>> = fs::read_to_string("graphs/01.edges")?
        .lines()
        .filter(|s| !s.is_empty())
        .map(parse_edge)
        .collect();
    let mut edges = edges?;

    let result = bhandari(&mut edges, 1, 5, 3).context("getting path from 1 to 5")?;

    println!("{result:#?}");

    Ok(())
}

fn bhandari(_graph: &[Edge], start: u32, to: u32, k: usize) -> anyhow::Result<Vec<Vec<u32>>> {
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
        let (shortest_path, cost) =
            dijkstra(&start, successors, |current_node| *current_node == to)
                .context("this graph doesn't contain such path")?;

        shortest_path
    };

    let mut paths: Vec<Vec<u32>> = Vec::with_capacity(k);
    paths.push(shortest_path);

    for i in 0..(k - 1) {
        // if node-disjoint path split the intermediate nodes of all Px where x < i
        // we use link-disjoint, so skip

        // Replace each link of all P_x where x < i with a reverse link of inverted link weight in the original graph
        let mut graph = _graph.to_vec();
        for path in &paths {
            for link in path.windows(2) {
                let from = link[0];
                let to = link[1];

                let link = graph
                    .iter_mut()
                    .find(|link| link.from == from && link.to == to)
                    .expect("link should be present");
                link.from = to;
                link.to = from;
                link.weight = -link.weight;
            }
        }

        // Find the shortest path Pi from node s to node t
        let successors = |current_node: &u32| {
            graph
                .iter()
                .filter(|edge| edge.from == *current_node)
                .map(|&Edge { to, weight, .. }| (to, weight))
                .collect::<Vec<_>>()
        };
        let (shortest_path, _cost) =
            dijkstra(&start, successors, |current_node| *current_node == to)
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

        for path in &mut paths {
            let links = path.windows(2).map(|l| (l[0], l[1])).collect::<Vec<_>>();
            for link in links {
                if !unique_links.contains(&link) {
                    let to = link.1;
                    let pos = path
                        .iter()
                        .position(|node| *node == to)
                        .expect("path should contain node");
                    path.remove(pos);
                }
            }
        }
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

    let edge = parts.next().context("no edge")?;
    let (weight, rest) = edge
        .split_once(|c: char| !c.is_ascii_digit())
        .context("no weight")?;
    ensure!(rest == ">");
    let weight = weight.parse()?;

    let to = parts.next().context("no finish node")?.parse()?;

    Ok(Edge { from, to, weight })
}

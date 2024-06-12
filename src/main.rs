use clap::{arg, command, Parser};
use pathfinding::prelude::*;
use std::{collections::HashMap, fs, sync::Arc};

use anyhow::Context;

#[derive(Debug, Clone, Parser)]
#[command()]
struct Args {
    #[arg()]
    file: String,

    #[arg()]
    start: String,

    #[arg()]
    to: String,

    #[arg()]
    k: usize,

    /// Treat links as undirected, false by default
    #[arg(short, long)]
    undirected: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let edges =
        load_edges_from_file(&args.file, args.undirected).context("loading edges from file")?;

    let result =
        bhandari(&edges, &args.start, &args.to, args.k).context("getting disjoint paths")?;

    println!("{result:?}");

    Ok(())
}

fn bhandari(_graph: &[Edge], start: &str, end: &str, k: usize) -> anyhow::Result<Vec<Vec<String>>> {
    struct Edge {
        from: u32,
        to: u32,
        weight: i32,
    }

    // convert string nodes to numbers
    let mut nodes = _graph
        .iter()
        .map(|link| link.from.clone())
        .chain(_graph.iter().map(|link| link.to.clone()))
        .collect::<Vec<_>>();
    nodes.sort();
    nodes.dedup();

    let nodes_names_to_indices: HashMap<String, u32> = HashMap::from_iter(
        nodes
            .iter()
            .enumerate()
            .map(|(i, s)| (s.to_string(), u32::try_from(i).unwrap())),
    );
    let nodes_indices_to_names = nodes;

    let graph = _graph
        .iter()
        .map(|edge| Edge {
            from: *nodes_names_to_indices.get(&*edge.from).unwrap(),
            to: *nodes_names_to_indices.get(&*edge.to).unwrap(),
            weight: edge.weight,
        })
        .collect::<Vec<_>>();

    let start = *nodes_names_to_indices.get(start).unwrap();
    let end = *nodes_names_to_indices.get(end).unwrap();

    // dijkstra calls a function at each step to get list of next nodes it goes to, so transform our
    // edge list to lambda that returns `to` nodes for a given node
    let shortest_path = {
        let successors = |current_node: &u32| {
            graph
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
        let mut graph: HashMap<(u32, u32), i32> =
            HashMap::from_iter(graph.iter().map(|edge| ((edge.from, edge.to), edge.weight)));

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

    // restore original node names
    let paths: Vec<Vec<String>> = paths
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|node| nodes_indices_to_names[node as usize].to_string())
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(paths)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Edge {
    from: Arc<str>,
    to: Arc<str>,
    weight: i32,
}

fn load_edges_from_file(file: &str, undirected: bool) -> anyhow::Result<Vec<Edge>> {
    let edges = fs::read_to_string(file)?
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .flat_map(|line| parse_edge(line, undirected).unwrap())
        .collect();

    Ok(edges)
}

fn parse_edge(line: &str, undirected: bool) -> anyhow::Result<Vec<Edge>> {
    let mut parts = line.split_whitespace();

    let from: Arc<str> = parts.next().context("no starting node")?.into();
    let weight = parts.next().context("no weight")?.parse()?;
    let to: Arc<str> = parts.next().context("no finish node")?.into();

    Ok(if undirected {
        vec![
            Edge {
                from: from.clone(),
                to: to.clone(),
                weight,
            },
            Edge {
                from: to,
                to: from,
                weight,
            },
        ]
    } else {
        vec![Edge { from, to, weight }]
    })
}

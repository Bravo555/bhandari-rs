use std::{fs, sync::Arc};

use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: Arc<str>,
    pub to: Arc<str>,
    pub weight: i32,
}

pub fn load_edges_from_file(file: &str, undirected: bool) -> anyhow::Result<Vec<Edge>> {
    let edges = fs::read_to_string(file)?
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .flat_map(|line| parse_edge(line, undirected).unwrap())
        .collect();

    Ok(edges)
}

pub fn parse_edge(line: &str, undirected: bool) -> anyhow::Result<Vec<Edge>> {
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

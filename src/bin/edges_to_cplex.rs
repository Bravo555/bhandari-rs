use std::{fmt::Write, fs, ops::Deref, sync::Arc};

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let links = bhandari_rs::load_edges_from_file("graphs/wikipedia_links.edges", true)
        .context("loading graph from file")?;

    let mut nodes: Vec<Arc<str>> = links.iter().map(|l| l.from.clone()).collect();

    // when undirectional, (from, to) and (to, from) pairs are inserted next to each other, need to sort by `from`
    nodes.sort();
    nodes.dedup();

    println!("{:?}", &nodes[0..10]);
    println!("len: {}", nodes.len());

    let n = nodes.len();

    let mut distance_matrix = vec![vec![999; n]; n];

    for link in links {
        let from_idx = nodes
            .binary_search(&link.from)
            .expect("node should be present");
        let to_idx = nodes
            .binary_search(&link.to)
            .expect("node should be present");

        distance_matrix[from_idx][to_idx] = link.weight;
    }

    let mut output = String::new();

    writeln!(&mut output, "n = {};", nodes.len())?;

    writeln!(
        &mut output,
        "source = {};",
        nodes
            .binary_search_by(|n| n.deref().cmp("Adolf_Hitler"))
            .unwrap()
    )?;

    writeln!(
        &mut output,
        "target = {};",
        nodes.binary_search_by(|n| n.deref().cmp("Emacs")).unwrap()
    )?;

    writeln!(&mut output, "K = 2;",)?;

    writeln!(&mut output)?;

    writeln!(&mut output, "distance=[")?;

    for row in distance_matrix {
        writeln!(&mut output, "{row:?},")?;
    }

    writeln!(&mut output, "];")?;

    fs::write("graph-wikipedia-links.dat", output)?;

    Ok(())
}

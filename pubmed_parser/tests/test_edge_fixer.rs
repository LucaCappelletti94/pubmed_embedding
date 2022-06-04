extern crate pubmed_parser;
use indicatif::ProgressBar;
use indicatif::ProgressIterator;
use std::{
    collections::HashSet,
    io::{BufRead, Write},
};

#[test]
fn test_edge_fixer() {
    let nodes = std::fs::File::open("/bfd/pubmed/tsv/cleaned_nodes.tsv").unwrap();
    let nodes = std::io::BufReader::new(nodes);

    let pb = ProgressBar::new(33_722_732 as u64);

    let unique_nodes: HashSet<String> = nodes
        .lines()
        .progress_with(pb)
        .map(|line| {
            let line = line.unwrap();
            let splits = line.splitn(3, '\t').collect::<Vec<&str>>();
            splits[0].to_string()
        })
        .collect();

    let source = std::fs::File::open("/bfd/pubmed/tsv/cleaned_edges.tsv").unwrap();
    let source = std::io::BufReader::new(source);

    let destination = std::fs::File::create("/bfd/pubmed/tsv/really_cleaned_edges.tsv").unwrap();
    let mut destination = std::io::BufWriter::new(destination);

    let pb = ProgressBar::new(674_342_790 as u64);

    source.lines().progress_with(pb).for_each(|line| {
        let original = line.unwrap();
        let splits = original.splitn(3, '\t').collect::<Vec<&str>>();
        if !splits[0].is_empty()
            && !splits[2].is_empty()
            && unique_nodes.contains(splits[0])
            && unique_nodes.contains(splits[2])
        {
            destination.write(original.as_bytes()).unwrap();
        }
    });
}

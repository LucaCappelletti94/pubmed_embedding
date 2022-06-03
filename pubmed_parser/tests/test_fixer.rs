extern crate pubmed_parser;
use indicatif::ProgressBar;
use indicatif::ProgressIterator;
use std::{
    collections::HashSet,
    io::{BufRead, Write},
};

#[test]
fn test_fixer() {
    let mut unique_nodes: HashSet<String> = HashSet::new();

    let source = std::fs::File::open("/bfd/pubmed/tsv/nodes.tsv").unwrap();
    let source = std::io::BufReader::new(source);

    let destination = std::fs::File::open("/bfd/pubmed/tsv/cleaned_nodes.tsv").unwrap();
    let mut destination = std::io::BufWriter::new(destination);

    let pb = ProgressBar::new(512_762_623 as u64);

    source.lines().progress_with(pb).for_each(|line| {
        let line = line.unwrap();
        let splits = line.rsplitn(2, '\t').collect::<Vec<&str>>();
        if unique_nodes.insert(splits[0].to_string()) {
            destination
                .write(
                    format!(
                        "{}\t{}\t{}\n",
                        splits[0],
                        splits[1],
                        splits[2].replace('\t', " ")
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    });
}

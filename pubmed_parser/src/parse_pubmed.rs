use crate::article::*;
use crate::article_builder::*;
use flate2::read::GzDecoder;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressIterator;
use std::fs::File;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::io::Write;
use std::io::BufWriter;
use std::fs;
use std::io::BufRead;

pub fn parse_single_pubmed(path: String) -> Vec<Result<Article, String>> {
    let file = std::fs::File::open(&path).unwrap();
    let file = GzDecoder::new(file);
    let file = std::io::BufReader::new(file);

    let mut article_builder = ArticleBuilder::new();
    file.lines()
        .filter_map(|line| match line {
            Ok(line) => {
                let line = line.trim();
                if [
                    "<?xml",
                    "<?nihms ?>",
                    "<?pmcsd ?>",
                    "<!DOCTYPE",
                    "<PubmedArticleSet>",
                    "<PubmedData>",
                    "</PubmedData>",
                    "<PublicationStatus>",
                    "</PubmedArticleSet>",
                    "<MedlineCitation",
                    "</MedlineCitation",
                    "<CitationSubset>",
                    "<Article ",
                    "<ArticleTitle/>",
                    "<PublicationTypeList/>",
                    "<ReferenceList/>",
                    "</Article>",
                    "<NumberOfReferences>",
                    "<Language>",
                    "<SpaceFlightMission>",
                    "<OtherID ",
                    "<ELocationID ",
                    "<GeneralNote ",
                    "<Pagination",
                    "</Pagination",
                    "<MedlinePgn",
                ]
                .iter()
                .any(|target| line.starts_with(target) || line.ends_with(target))
                {
                    return None;
                }
                article_builder
                    .parse(line)
                    .map_err(|err| format!("{} {}", err, path))
                    .unwrap();
                if article_builder.can_build() {
                    Some(Ok(core::mem::replace(
                        &mut article_builder,
                        ArticleBuilder::new(),
                    )
                    .build()
                    .unwrap()))
                } else {
                    None
                }
            }
            Err(_) => Some(Err(format!("Failed with decompression of file {}.", path))),
        })
        .collect::<Vec<Result<_, _>>>()
}

pub fn parse_pubmed(directory: &str) {
    let paths = fs::read_dir(directory)
        .unwrap()
        .map(|path| path.unwrap().path().display().to_string())
        .filter(|path| path.ends_with(".gz"))
        .collect::<Vec<String>>();

    let pb = ProgressBar::new(paths.len() as u64);

    let edges = File::create("edges.tsv").unwrap();
    let nodes = File::create("nodes.tsv").unwrap();

    let mut edges = BufWriter::new(edges);

    edges.write(b"subject\tedge_type\tobject").unwrap();

    let mut nodes = BufWriter::new(nodes);

    nodes.write(b"node_name\tnode_type\tdescription").unwrap();

    paths
        .into_iter()
        .progress_with(pb)
        .flat_map(parse_single_pubmed)
        .for_each(|article|{
            let article = article.unwrap();
            for node in article.to_nodes() {
                nodes.write(format!(
                    "{}\t{}\t{}",
                    node.node_name,
                    node.node_type,
                    node.description,
                ).as_bytes()).unwrap();
            }
            for edge in article.to_edges() {
                edges.write(format!(
                    "{}\t{}\t{}",
                    edge.subject,
                    edge.edge_type,
                    edge.object,
                ).as_bytes()).unwrap();
            }
        });
}

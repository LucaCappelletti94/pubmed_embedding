use crate::article::*;
use crate::article_builder::*;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use rayon::prelude::*;
use std::fs;
use flate2::read::GzDecoder;
use std::io::BufRead;

pub fn parse_single_pubmed(path: String) -> Vec<Result<Article, std::io::Error>> {
    let file = std::fs::File::open(path).unwrap();
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
                    "<CoiStatement>",
                    "</Article>",
                    "<NumberOfReferences>",
                    "<Language>",
                    "<SpaceFlightMission>",
                    "<ArticleIdList>",
                    "<OtherID ",
                    "<ELocationID ",
                    "<GeneralNote ",
                    "<VernacularTitle>",
                    "</ArticleIdList>",
                    "<Pagination",
                    "</Pagination",
                    "<MedlinePgn",
                ]
                .iter()
                .any(|starter| line.starts_with(starter))
                {
                    return None;
                }
                article_builder.parse(line).unwrap();
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
            Err(line) => Some(Err(line)),
        })
        .collect::<Vec<Result<_, _>>>()
}
pub fn parse_pubmed(directory: &str) -> Result<Vec<Article>, std::io::Error> {
    let paths = fs::read_dir(directory)
        .unwrap()
        .map(|path| path.unwrap().path().display().to_string())
        .filter(|path| path.ends_with(".gz"))
        .collect::<Vec<String>>();

    let pb = ProgressBar::new(paths.len() as u64);

    paths
        .into_par_iter()
        .progress_with(pb)
        .flat_map(parse_single_pubmed)
        .collect()
}

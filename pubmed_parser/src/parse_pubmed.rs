use crate::article::*;
use crate::article_builder::*;
use flate2::read::GzDecoder;
use indicatif::ParallelProgressIterator;
use indicatif::ProgressBar;
use rayon::prelude::*;
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
            Err(line) => Some(Err(format!("Failed with decompression of file {}.", path))),
        })
        .collect::<Vec<Result<_, _>>>()
}
pub fn parse_pubmed(directory: &str) -> Result<Vec<Article>, String> {
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

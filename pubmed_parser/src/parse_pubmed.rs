use crate::article::*;
use crate::article_builder::*;
use rayon::prelude::*;
use std::io::BufRead;

pub fn parse_single_pubmed(path: &str) -> Vec<Result<Article, std::io::Error>> {
    let file = std::fs::File::open(path).unwrap();
    let file = std::io::BufReader::new(file);
    let mut article_builder = ArticleBuilder::new();
    file.lines()
        .filter_map(|line| match line {
            Ok(line) => {
                let line = line.trim();
                if [
                    "<?xml",
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
                    Some(
                        Ok(core::mem::replace(&mut article_builder, ArticleBuilder::new())
                            .build()
                            .unwrap()),
                    )
                } else {
                    None
                }
            }
            Err(line) => Some(Err(line)),
        })
        .collect::<Vec<Result<_, _>>>()
}
pub fn parse_pubmed() -> Result<Vec<Article>, std::io::Error> {
    // TODO! make tools that reads the files from a given directory.

    let paths = vec!["../pubmed22n0012.xml"];
    paths
        .into_par_iter()
        .flat_map(parse_single_pubmed)
        .collect()
}

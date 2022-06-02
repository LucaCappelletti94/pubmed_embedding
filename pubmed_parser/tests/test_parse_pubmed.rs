extern crate pubmed_parser;

#[test]
fn test_pubmed_parser() {
    let result = pubmed_parser::parse_pubmed("../data/");
    assert!(result.is_ok());
}

extern crate pubmed_parser;

#[test]
fn test_pubmed_bfd_parser() {
    let result = pubmed_parser::parse_pubmed(
        "/bfd/pubmed/"
    );
    assert!(result.is_ok());
}
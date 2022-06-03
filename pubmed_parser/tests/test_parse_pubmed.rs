extern crate pubmed_parser;

#[test]
fn test_pubmed_parser() {
    pubmed_parser::parse_pubmed(
        "../data/"
    );
}

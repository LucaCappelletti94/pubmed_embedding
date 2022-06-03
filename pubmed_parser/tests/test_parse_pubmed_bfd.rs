extern crate pubmed_parser;

#[test]
fn test_pubmed_bfd_parser() {
    pubmed_parser::parse_pubmed(
        "/bfd/pubmed/"
    );
}

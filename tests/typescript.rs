use std::io::BufWriter;

use cddl::visitor::Visitor;
use cddlconv;

#[test]
fn it_works() {
    let input = std::fs::read_to_string("examples/webdriver-bidi/webdriver-bidi.cddl").unwrap();
    let cddl = cddl::parser::cddl_from_str(&input, true).unwrap();
    let stdout = BufWriter::new(Vec::new());
    let stderr = BufWriter::new(Vec::new());
    let mut engine = cddlconv::engines::typescript::Engine::with_writers(stdout, stderr);
    engine.visit_cddl(&cddl).unwrap();
    engine.print_postamble();

    let (stdout, stderr) = engine.into_writers();
    insta::assert_snapshot!(String::from_utf8(stderr.into_inner().unwrap()).unwrap());
    insta::assert_snapshot!(String::from_utf8(stdout.into_inner().unwrap()).unwrap());
}

#[test]
fn it_works_with_arrays() {
    let input = std::fs::read_to_string("examples/rfc-examples/arrays.cddl").unwrap();
    let cddl = cddl::parser::cddl_from_str(&input, true).unwrap();
    let stdout = BufWriter::new(Vec::new());
    let stderr = BufWriter::new(Vec::new());
    let mut engine = cddlconv::engines::typescript::Engine::with_writers(stdout, stderr);
    engine.visit_cddl(&cddl).unwrap();
    engine.print_postamble();

    let (stdout, stderr) = engine.into_writers();
    insta::assert_snapshot!(String::from_utf8(stderr.into_inner().unwrap()).unwrap());
    insta::assert_snapshot!(String::from_utf8(stdout.into_inner().unwrap()).unwrap());
}

#[test]
fn it_works_with_maps() {
    let input = std::fs::read_to_string("examples/rfc-examples/maps.cddl").unwrap();
    let cddl = cddl::parser::cddl_from_str(&input, true).unwrap();
    let stdout = BufWriter::new(Vec::new());
    let stderr = BufWriter::new(Vec::new());
    let mut engine = cddlconv::engines::typescript::Engine::with_writers(stdout, stderr);
    engine.visit_cddl(&cddl).unwrap();
    engine.print_postamble();

    let (stdout, stderr) = engine.into_writers();
    insta::assert_snapshot!(String::from_utf8(stderr.into_inner().unwrap()).unwrap());
    insta::assert_snapshot!(String::from_utf8(stdout.into_inner().unwrap()).unwrap());
}

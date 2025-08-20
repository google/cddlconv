use std::io::BufWriter;

use cddl::visitor::Visitor;
use cddlconv;

macro_rules! test {
    ($name:ident, $input:expr) => {
        #[test]
        fn $name() {
            let input = std::fs::read_to_string($input).unwrap();
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
    };
}

test!(it_works, "examples/webdriver-bidi/webdriver-bidi.cddl");
test!(it_works_with_arrays, "examples/rfc-examples/arrays.cddl");
test!(it_works_with_maps, "examples/rfc-examples/maps.cddl");
test!(
    it_works_with_amendments,
    "examples/rfc-examples/colors.cddl"
);
test!(it_works_with_optional_groups, "examples/optional_groups.cddl");
test!(it_works_with_simple_optional_groups, "examples/simple_optional_groups.cddl");
test!(it_works_with_array_occurences, "examples/array_occurences.cddl");

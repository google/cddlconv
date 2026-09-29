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
            let mut engine = cddlconv::engines::zod::Engine::with_writers(stdout, stderr);
            engine.visit_cddl(&cddl).unwrap();
            engine.print_postamble();

            let (stdout, stderr) = engine.into_writers();
            insta::assert_snapshot!(String::from_utf8(stderr.into_inner().unwrap()).unwrap());
            insta::assert_snapshot!(String::from_utf8(stdout.into_inner().unwrap()).unwrap());
        }
    };
}

test!(it_works, "examples/webdriver-bidi/webdriver-bidi.cddl");
test!(it_works_with_maps, "examples/rfc-examples/maps.cddl");
test!(
    it_works_with_amendments,
    "examples/rfc-examples/colors.cddl"
);
test!(
    it_works_with_prelude_text_types,
    "examples/rfc-examples/prelude_text_types.cddl"
);
test!(
    it_works_with_optional_groups,
    "examples/optional_groups.cddl"
);
test!(
    it_works_with_simple_optional_groups,
    "examples/simple_optional_groups.cddl"
);
test!(
    it_works_with_array_occurences,
    "examples/array_occurences.cddl"
);
test!(it_works_with_enums, "examples/enums.cddl");

#[test]
fn zod4_uses_v4_type_annotations() {
    let input = std::fs::read_to_string("examples/zod4_compat.cddl").unwrap();
    let cddl = cddl::parser::cddl_from_str(&input, true).unwrap();
    let stdout = BufWriter::new(Vec::new());
    let stderr = BufWriter::new(Vec::new());
    let mut engine = cddlconv::engines::zod::Engine::with_writers_zod4(stdout, stderr);

    engine.visit_cddl(&cddl).unwrap();
    engine.print_postamble();

    let (stdout, stderr) = engine.into_writers();
    let stdout = String::from_utf8(stdout.into_inner().unwrap()).unwrap();
    let stderr = String::from_utf8(stderr.into_inner().unwrap()).unwrap();

    assert!(stderr.is_empty());
    assert!(stdout.contains("z.ZodUnion<readonly [z.ZodString,z.ZodNumber]>"));
    assert!(stdout.contains(
        r#"z.ZodEnum<{"decline":"decline";"dedicated-worker":"dedicated-worker";"-0":"-0";}>"#
    ));
    assert!(stdout.contains("z.ZodArray<z.ZodString>"));
    assert!(!stdout.contains(r#""atleastone""#));
}

#[test]
fn default_zod_output_remains_v3_compatible() {
    let input = std::fs::read_to_string("examples/zod4_compat.cddl").unwrap();
    let cddl = cddl::parser::cddl_from_str(&input, true).unwrap();
    let stdout = BufWriter::new(Vec::new());
    let stderr = BufWriter::new(Vec::new());
    let mut engine = cddlconv::engines::zod::Engine::with_writers(stdout, stderr);

    engine.visit_cddl(&cddl).unwrap();

    let (stdout, _) = engine.into_writers();
    let stdout = String::from_utf8(stdout.into_inner().unwrap()).unwrap();

    assert!(stdout.contains("z.ZodUnion<[z.ZodString,z.ZodNumber]>"));
    assert!(stdout.contains(r#"z.ZodEnum<["decline","dedicated-worker","-0",]>"#));
    assert!(stdout.contains(r#"z.ZodArray<z.ZodString, "atleastone">"#));
}

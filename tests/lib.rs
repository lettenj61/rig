extern crate vtol;

mod format_test {

    use vtol::format::format;

    const W: &'static str = "Fabulous Is Rust";

    #[test]
    fn lower_case() {
        assert_eq!(format(W, "lower".into()), "fabulous is rust");
    }

    #[test]
    fn upper_case() {
        assert_eq!(format(W, "upper".into()), "FABULOUS IS RUST");
    }

    #[test]
    fn capitalize() {
        assert_eq!(format(&W.to_lowercase(), "cap".into()), "Fabulous is rust");
    }

    #[test]
    fn decapitalize() {
        assert_eq!(format(W, "decap".into()), "fabulous Is Rust");
    }

    #[test]
    fn start_case() {
        assert_eq!(format(&W.to_lowercase(), "start".into()), W);
    }

    #[test]
    fn word_only() {
        let includes_ctrls = "_!$what'is-this@charac#ters??";
        assert_eq!(format(includes_ctrls, "word".into()), "_whatisthischaracters");
    }

    #[test]
    fn hyphenate() {
        assert_eq!(format(W, "hyphen".into()), "Fabulous-Is-Rust");
    }

    #[test]
    fn upper_camel() {
        assert_eq!(format(W, "Camel".into()), "FabulousIsRust");
    }

    #[test]
    fn lower_camel() {
        assert_eq!(format(W, "camel".into()), "fabulousIsRust");
    }

    #[test]
    fn normalize() {
        assert_eq!(format(W, "norm".into()), "fabulous-is-rust");
    }

    #[test]
    fn snake_case() {
        assert_eq!(format(W, "snake".into()), "Fabulous_Is_Rust");
    }

    #[test]
    fn directory_path() {
        let p = "path.to.my.directory";
        assert_eq!(format(p, "packaged".into()), "path/to/my/directory");
    }
}

mod template_test {

    use std::collections::HashMap;
    use vtol::template::{self, Template};

    use std::io;

    #[test]
    fn compile_inline() {
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("name".to_owned(), "Rust".to_owned());

        let mut out = Vec::new();

        Template::compile_inline("Hello, $name$!", &mut out, ctx).unwrap();
        assert_eq!(::std::str::from_utf8(&out).unwrap(), "Hello, Rust!");
    }
}

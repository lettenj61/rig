extern crate rig;

mod format_test {

    use std::collections::HashMap;
    use rig::format::{format, Placeholder};

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
        assert_eq!(format(includes_ctrls, "word".into()),
                   "_whatisthischaracters");
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

    #[test]
    fn placeholder_giter8() {
        let ph = Placeholder::parse_g8(r#"project_name;format="upper,hyphen""#).unwrap();
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("project_name".to_owned(), "A simple proj".to_owned());

        assert_eq!(ph.format_with(&ctx), "A-SIMPLE-PROJ".to_owned());
    }

    #[test]
    fn placeholder_simple() {
        let ph = Placeholder::parse_simple("project_name;norm").unwrap();
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("project_name".to_owned(),
                   "A si\nmPLe         p\trOj".to_owned());

        assert_eq!(ph.format_with(&ctx), "a-si-mple-p-roj".to_owned());
    }

    #[test]
    fn placeholder_dirname() {
        let ph = Placeholder::parse_dirname("twitter_id__word_cap").unwrap();
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("twitter_id".to_owned(), "bar3s%Ye".to_owned());

        assert_eq!(ph.format_with(&ctx), "Bar3sye".to_owned());
    }
}

mod template_test {

    use std::collections::HashMap;
    use std::str;
    use rig::template::Template;

    #[test]
    fn compile_inline() {
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("name".to_owned(), "Rust".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out, "Hello, $name$!", ctx).unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(), "Hello, Rust!");
    }

    #[test]
    fn format_template() {
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("DOCUMENT_NAME".to_owned(),
                   "RUST PROGRAMMING LANGUAGE".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out, "It's a $DOCUMENT_NAME;camel$", ctx).unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(),
                   "It's a rustProgrammingLanguage".to_owned());
    }

    #[test]
    fn escape_character() {
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("DOCUMENT_NAME".to_owned(),
                   "RUST PROGRAMMING LANGUAGE".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out, "It's a \\$DOCUMENT_NAME;camel$", ctx).unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(),
                   "It's a \\$DOCUMENT_NAME;camel$".to_owned());
    }

    #[test]
    fn giter8_template() {
        let mut ctx: HashMap<String, String> = HashMap::new();
        ctx.insert("name".to_owned(),
                   "awesome distributed interface".to_owned());

        let mut out = Vec::new();

        let mut tpl =
            Template::new_g8(r#"trait $name;format="Camel"$[-A] extends js.Dictionary[A]"#);
        tpl.write(&mut out, ctx).unwrap();

        assert_eq!(str::from_utf8(&out).unwrap(),
                   "trait AwesomeDistributedInterface[-A] extends js.Dictionary[A]".to_owned());
    }
}

mod repos_test {

    extern crate tempdir;

    use std::ffi;
    use std::path::Path;
    use self::tempdir::TempDir;

    #[test]
    fn path_sep() {

        let td = TempDir::new("sample-run").unwrap();
        let mut p = td.into_path();
        p.push("such/file");

        assert_eq!(format!("{:?}", p.file_name().unwrap()),
                   r#""file""#.to_string());
    }
}

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
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("project_name".to_owned(), "A simple proj".to_owned());

        assert_eq!(ph.format_with(&params), "A-SIMPLE-PROJ".to_owned());
    }

    #[test]
    fn placeholder_simple() {
        let ph = Placeholder::parse_simple("project_name;norm").unwrap();
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("project_name".to_owned(),
                      "A si\nmPLe         p\trOj".to_owned());

        assert_eq!(ph.format_with(&params), "a-si-mple-p-roj".to_owned());
    }

    #[test]
    fn placeholder_dirname() {
        let ph = Placeholder::parse_dirname("twitter_id__word_cap").unwrap();
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("twitter_id".to_owned(), "bar3s%Ye".to_owned());

        assert_eq!(ph.format_with(&params), "Bar3sye".to_owned());
    }
}

mod template_test {

    use std::collections::HashMap;
    use std::str;
    use rig::format::Style;
    use rig::template::Template;

    #[test]
    fn compile_inline() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("name".to_owned(), "Rust".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out, Style::Simple, "Hello, $name$!", &params).unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(), "Hello, Rust!");
    }

    #[test]
    fn format_template() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("DOCUMENT_NAME".to_owned(),
                      "RUST PROGRAMMING LANGUAGE".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out,
                                 Style::Simple,
                                 "It's a $DOCUMENT_NAME;camel$",
                                 &params)
            .unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(),
                   "It's a rustProgrammingLanguage".to_owned());
    }

    #[test]
    fn escape_character() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("DOCUMENT_NAME".to_owned(),
                      "RUST PROGRAMMING LANGUAGE".to_owned());

        let mut out = Vec::new();

        Template::compile_inline(&mut out,
                                 Style::Simple,
                                 "It's a \\$DOCUMENT_NAME;camel$",
                                 &params)
            .unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(),
                   "It's a \\$DOCUMENT_NAME;camel$".to_owned());
    }

    #[test]
    fn giter8_template() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("name".to_owned(),
                      "awesome distributed interface".to_owned());

        let mut out = Vec::new();

        let mut tpl =
            Template::new_g8(r#"trait $name;format="Camel"$[-A] extends js.Dictionary[A]"#);
        tpl.write(&mut out, &params).unwrap();

        assert_eq!(str::from_utf8(&out).unwrap(),
                   "trait AwesomeDistributedInterface[-A] extends js.Dictionary[A]".to_owned());
    }
}

mod project_test {

    extern crate tempdir;
    use std::fs;

    use rig::fsutils;
    use rig::project::{ConfigFormat, Project};

    const G8_PROPS: &'static str = r#"
        name = value1
        bar = baz!
        package = com.example.me
    "#;

    const TOML: &'static str = r#"
        name = "My Project"
        package = "deep.pkg.path"
        will_be_ignored = [4, 5, 6, 7]
        module_name = "quux"
    "#;

    #[test]
    fn simple_project() {

        let rust_dirs = vec![
            "src/sample",
            "src/sample/$package$",
            "ci"
        ];

        let src = tempdir::TempDir::new("rig-simple-test").unwrap();
        let src = src.path();
        for dir in &rust_dirs {
            let dir = src.join(dir);
            fs::create_dir_all(dir).unwrap();
        }

        let toml = src.join("_rig.toml");
        fsutils::write_file(&toml, TOML).unwrap();
        assert!(fsutils::exists(&toml));

        let dest = tempdir::TempDir::new("generated-proj").unwrap();
        let dest = dest.path();

        let project = Project::new(None as Option<&str>, ConfigFormat::Toml, false);
        let params = project.default_params(&src).unwrap();
        assert_eq!(params.get("name"), Some(&"My Project".to_owned()));
        assert_eq!(params.get("module_name"), Some(&"quux".to_owned()));
        assert!(params.get("will_be_ignored").is_none());

        project.generate(&params, &src, &dest, false).unwrap();

        let expected = vec![
            "ci",
            "src/sample",
            "src/sample/deep.pkg.path"
        ];

        for goal in &expected {
            let goal = &dest.join(goal);
            assert!(fsutils::exists(&goal));
        }
    }

    #[test]
    fn giter8_project() {

        let g8_dirs = vec![
            "src/main/g8",
            "src/main/g8/project",
            "src/main/g8/src/main/scala/$package$"
        ];

        let src = tempdir::TempDir::new("rig-g8-test").unwrap();
        let src = src.path();
        for dir in &g8_dirs {
            let dir = src.join(dir);
            fs::create_dir_all(dir).unwrap();
        }

        let props = src.join("src/main/g8/default.properties");
        fsutils::write_file(&props, G8_PROPS).unwrap();
        assert!(fsutils::exists(&props));

        let dest = tempdir::TempDir::new("generated-proj").unwrap();
        let dest = dest.path();

        let project = Project::new_g8(Some("src/main/g8"));

        let params = project.default_params(&src).unwrap();
        assert_eq!(params.get("name"), Some(&"value1".to_owned()));

        project.generate(&params, &src, &dest, false).unwrap();

        let expected = vec![
            "project",
            "src/main/scala/com",
            "src/main/scala/com/example",
            "src/main/scala/com/example/me",
        ];

        for goal in &expected {
            let goal = &dest.join(goal);
            assert!(fsutils::exists(&goal));
        }
    }
}

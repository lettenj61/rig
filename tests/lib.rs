extern crate rig;

mod format_test {

    use std::ascii::AsciiExt;
    use rig::format::format;

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
    fn add_random() {
        let len = W.len();
        let added = format(W, "random".into());
        assert_eq!(added.len(), len + 33); // 32 bytes of random chars + '-'
        for c in added[32..].chars() {
            assert!(c.is_alphanumeric() && c.is_ascii());
        }
    }
}

mod template_test {

    use std::collections::HashMap;
    use std::str;
    use rig::template::*;

    #[test]
    fn inline_giter8() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("project_name".to_owned(), "A simple proj".to_owned());
        let mut out = Vec::new();
        Template::write_once(&mut out,
                             Style::ST,
                             "$project_name;format=\"norm,upper\"$",
                             &params)
                             .unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(), "A-SIMPLE-PROJ".to_owned());
    }

    #[test]
    fn inline_path() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("twitter_id".to_owned(), "bar3s%Ye".to_owned());
        let mut out = Vec::new();
        Template::write_once(&mut out, Style::Path, "$twitter_id__word_cap$", &params).unwrap();

        assert_eq!(str::from_utf8(&out).unwrap(), "Bar3sye".to_owned());
    }

    #[test]
    fn write_once() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("name".to_owned(), "Rust".to_owned());

        let mut out = Vec::new();

        Template::write_once(&mut out, Style::ST, "Hello, $name$!", &params).unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(), "Hello, Rust!");
    }

    #[test]
    fn escape_character() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("DOCUMENT_NAME".to_owned(),
                      "RUST PROGRAMMING LANGUAGE".to_owned());

        let mut out = Vec::new();

        Template::write_once(&mut out,
                             Style::ST,
                             "It's a \\$DOCUMENT_NAME\\$",
                             &params)
            .unwrap();
        assert_eq!(str::from_utf8(&out).unwrap(),
                   "It's a $DOCUMENT_NAME$".to_owned());
    }

    #[test]
    fn giter8_template() {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("name".to_owned(),
                      "awesome distributed interface".to_owned());

        let mut out = Vec::new();

        let mut tpl =
            Template::new_g8(r#"trait $name;format="Camel"$[-A] extends js.Dictionary[A]"#);
        tpl.write_to(&mut out, &params).unwrap();

        assert_eq!(str::from_utf8(&out).unwrap(),
                   "trait AwesomeDistributedInterface[-A] extends js.Dictionary[A]".to_owned());
    }
}

mod project_test {

    extern crate tempdir;
    use std::fs;

    use rig::fsutils;
    use rig::project::{Configuration, Project};

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

        let toml = src.join("Rig.toml");
        fsutils::write_file(&toml, TOML).unwrap();
        assert!(fsutils::exists(&toml));

        let dest = tempdir::TempDir::new("generated-proj").unwrap();
        let dest = dest.path();

        let project = Project::new(None as Option<&str>, Configuration::Toml, false);
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

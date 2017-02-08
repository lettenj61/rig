use std::ascii::AsciiExt;
use std::convert::From;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Formatter {
    Ident,
    UpperCase,
    LowerCase,
    Capitalize,
    Decapitalize,
    StartCase,
    Hyphenate,
    WordChar,
    UpperCamel,
    LowerCamel,
    Normalize,
    SnakeCase,
    DirectoryPath,
    AddRandom,
}

impl<'a> From<&'a str> for Formatter {
    fn from(s: &str) -> Formatter {
        match s {
            "lower" | "lowercase" => Formatter::LowerCase,
            "upper" | "uppercase" => Formatter::UpperCase,
            "cap" | "capitalize" => Formatter::Capitalize,
            "decap" | "decapitalize" => Formatter::Decapitalize,
            "word" | "word-only" => Formatter::WordChar,
            "hyphen" | "hyphnate" => Formatter::Hyphenate,
            "start" | "start-case" => Formatter::StartCase,

            "Camel" | "upper-camel" => Formatter::UpperCamel,
            "camel" | "lower-camel" => Formatter::LowerCamel,

            "norm" | "normalize" => Formatter::Normalize,
            "snake" | "snake-case" => Formatter::SnakeCase,
            "packaged" | "package-dir" => Formatter::DirectoryPath,

            "random" |
            "generate-random" => Formatter::AddRandom,

            _ => Formatter::Ident,
        }
    }
}

fn process_words<F>(s: &str, f: F) -> String
    where F: FnMut(&str) -> String
{
    s.split_whitespace()
        .map(f)
        .collect::<Vec<_>>()
        .join(" ")
        .into()
}

fn join_camel_case(s: &str, pascal: bool) -> String {
    if let Some(_) = s.find(' ') {
        let mut words = s.split_whitespace();
        let head = match pascal {
            true => capitalize(words.next().unwrap()),
            false => (words.next().unwrap()).to_lowercase(),
        };
        let tail = words.map(capitalize).collect::<Vec<_>>().concat();
        word_chars_only(&(head + &tail))
    } else {
        word_chars_only(&s)
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => "".into(),
        Some(f) => f.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

fn decapitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => "".into(),
        Some(f) => f.to_lowercase().collect::<String>() + &chars.as_str(),
    }
}

fn word_chars_only(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii() && (*c == '_' || c.is_alphanumeric()))
        .collect::<String>()
}

fn normalize(s: &str) -> String {
    let s = dedup_whitespace(&s.to_lowercase());
    s.replace(" ", "-")
}

fn snake_case(s: &str) -> String {
    let mut s = dedup_replacing(s, '.', ' ');
    s = dedup_replacing(&s, '-', ' ');
    dedup_whitespace(&s).replace(" ", "_")
}

fn directory_path(s: &str) -> String {
    let s = dedup(s, '.');
    s.replace(".", "/")
}

fn dedup(s: &str, c: char) -> String {
    dedup_replacing(s, c, c)
}

fn dedup_replacing(s: &str, from: char, to: char) -> String {
    let mut dupe = String::new();
    for _ in 0..2 {
        dupe.push(from);
    }

    let mut s = s.to_string();
    while let Some(_) = s.find(&dupe) {
        s = s.replace(&dupe, &from.to_string());
    }

    s.replace(&from.to_string(), &to.to_string())
}

fn dedup_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// format a `&str` sentence to `String`.
pub fn format(s: &str, f: Formatter) -> String {
    match f {
        Formatter::LowerCase => s.to_lowercase(),
        Formatter::UpperCase => s.to_uppercase(),
        Formatter::Capitalize => capitalize(s),
        Formatter::Decapitalize => decapitalize(s),
        Formatter::StartCase => process_words(s, capitalize),
        Formatter::WordChar => process_words(s, word_chars_only),
        Formatter::Hyphenate => dedup_whitespace(s).replace(" ", "-"),
        Formatter::UpperCamel => join_camel_case(s, true),
        Formatter::LowerCamel => join_camel_case(s, false),
        Formatter::Normalize => normalize(s),
        Formatter::SnakeCase => snake_case(s),
        Formatter::DirectoryPath => directory_path(s),
        _ => s.into(),
    }
}

use std::ascii::AsciiExt;
use std::convert::From;

#[derive(Debug)]
pub enum Format {
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

impl<'a> From<&'a str> for Format {
    fn from(s: &str) -> Format {
        match s {
            "lower" | "lowercase" => Format::LowerCase,
            "upper" | "uppercase" => Format::UpperCase,
            "cap" | "capitalize" => Format::Capitalize,
            "decap" | "decapitalize" => Format::Decapitalize,
            "word" | "word-only" => Format::WordChar,
            "hyphen" | "hyphnate" => Format::Hyphenate,
            "start" | "start-case" => Format::StartCase,

            "pascal" | "Camel" | "upper-camel" => Format::UpperCamel,
            "camel" | "lower-camel" => Format::LowerCamel,

            "norm" | "normalize" => Format::Normalize,
            "snake" | "snake-case" => Format::SnakeCase,
            "packaged" | "package-dir" => Format::DirectoryPath,

            "random" |
            "generate-random" => Format::AddRandom,

            _ => Format::Ident,
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
            false => decapitalize(words.next().unwrap()),
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
    s.to_lowercase().replace(" ", "-")
}

fn snake_case(s: &str) -> String {
    let s = dedup_dots(s);
    s.replace(".", "_").replace(" ", "_")
}

fn directory_path(s: &str) -> String {
    let s = dedup_dots(s);
    s.replace(".", "/")
}

/// Deduplicate a dot char appears sequently.
fn dedup_dots(s: &str) -> String {
    let mut s = s.to_string();
    while let Some(_) = s.find("..") {
        s = s.replace("..", ".")
    }
    s
}

/// Format a `&str` sentence to `String`.
pub fn format(s: &str, f: Format) -> String {
    match f {
        Format::LowerCase => s.to_lowercase(),
        Format::UpperCase => s.to_uppercase(),
        Format::Capitalize => capitalize(s),
        Format::Decapitalize => decapitalize(s),
        Format::StartCase => process_words(s, capitalize),
        Format::WordChar => process_words(s, word_chars_only),
        Format::Hyphenate => s.replace(" ", "-"),
        Format::UpperCamel => join_camel_case(s, true),
        Format::LowerCamel => join_camel_case(s, false),
        Format::Normalize => normalize(s),
        Format::SnakeCase => snake_case(s),
        Format::DirectoryPath => directory_path(s),
        _ => s.into(),
    }
}

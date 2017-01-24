use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::convert::From;

#[derive(Clone, Debug)]
pub enum Style {
    Simple,
    Giter8,
}

impl Default for Style {
    fn default() -> Style {
        Style::Simple
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Default)]
pub struct Placeholder {
    key: String,
    args: Vec<Format>,
}

impl Placeholder {

    pub fn new(key: &str, args: Vec<&str>) -> Placeholder {
        Placeholder {
            key: key.into(),
            args: args.iter().map(|&s| Format::from(s)).filter(|f| *f != Format::Ident).collect(),
        }
    }

    pub fn no_format(key: &str) -> Placeholder {
        Placeholder::new(key, Vec::new())
    }

    /// Parse default style template format string to a `Placeholder`.
    pub fn parse_simple(expr: &str) -> Result<Placeholder, String> {
        let args = expr.split(';').collect::<Vec<_>>();
        match args.len() {
            0 => Err("Placeholder is empty.".into()),
            1 => Ok(Placeholder::no_format(expr)),
            2 => {
                let key = args[0];
                let formats = args[1].split(',').collect::<Vec<_>>();
                Ok(Placeholder::new(&key, formats))
            },
            _ => Err("Too many separators in placeholder.".into()),
        }
    }

    /// Parse giter8 style template format string to a `Placeholder`.
    pub fn parse_g8(expr: &str) -> Result<Placeholder, String> {
        let args = expr.split(';').collect::<Vec<_>>();
        match args.len() {
            0 => Err("Placeholder is empty.".into()),
            1 => Ok(Placeholder::no_format(expr)),
            2 => {
                let key = args[0];
                let raw_format = args[1].replace('"', "").replace("format=", "");
                let formats = raw_format.split(',').collect::<Vec<_>>();
                Ok(Placeholder::new(&key, formats))
            },
            _ => Err("Too many separators in placeholder.".into()),
        }
    }

    /// Parse template format as an directory or file name to a `Placeholder`.
    pub fn parse_dirname(expr: &str) -> Result<Placeholder, String> {
        let args = expr.split("__").collect::<Vec<_>>();
        match args.len() {
            0 => Err("File / directory name cannot be empty.".into()),
            2 => {
                let key = args[0];
                let formats = args[1]
                    .split('_')
                    .collect::<Vec<_>>();
                Ok(Placeholder::new(&key, formats))
            },
            _ => Ok(Placeholder::no_format(expr)), // ignore more than two separators
        }
    }

    /// Apply formatting on the placeholder with given context, and returns formatted `String`.
    pub fn format_with(&self, context: &HashMap<String, String>) -> String {
        if let Some(v) = context.get(&self.key) {
            self.args.iter().fold(v.clone(), |ref s, f| format(&s, *f))
        } else {
            self.key.clone()
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

    s.replace(&dupe, &to.to_string())
}

fn dedup_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
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
        Format::Hyphenate => dedup_whitespace(s).replace(" ", "-"),
        Format::UpperCamel => join_camel_case(s, true),
        Format::LowerCamel => join_camel_case(s, false),
        Format::Normalize => normalize(s),
        Format::SnakeCase => snake_case(s),
        Format::DirectoryPath => directory_path(s),
        _ => s.into(),
    }
}

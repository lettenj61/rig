use std::collections::HashMap;
use std::convert::From;
use std::io::{self, Write};
use std::path::Path;

use toml::{Table, Value};

use super::format::{Placeholder, Style};
use super::fsutils;

/// Minimal template for any kind of plain text.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    style: Style,
    body: String,
}

impl Template {
    /// Create `Template` object from given `str`.
    pub fn read_str<S: AsRef<str>>(style: Style, template: S) -> Template {
        Template {
            style: style,
            body: String::from(template.as_ref()),
        }
    }

    /// Create `Template` from contents of the file at given `Path`.
    pub fn read_file<P: AsRef<Path>>(style: Style, src: P) -> Result<Template, io::Error> {
        fsutils::read_file(src.as_ref()).map(|s| Template::read_str(style, s))
    }

    /// Utility to create giter8 style template instantly.
    pub fn new_g8<S: AsRef<str>>(template: S) -> Template {
        Template::read_str(Style::Giter8, template)
    }

    /// Create template from given `str`, and instantly compile it.
    pub fn compile_inline<'a, S, W>(writer: &'a mut W,
                                    style: Style,
                                    template: S,
                                    params: &HashMap<String, String>)
                                    -> Result<&'a mut W, io::Error>
        where S: AsRef<str>,
              W: Write
    {
        let mut template = Template::read_str(style, template);
        Template::write(&mut template, writer, params)
    }

    /// Replace all placeholders its holding with values from given params.
    fn process(&self, ph: &str, params: &HashMap<String, String>) -> Option<String> {

        let parsed = match self.style {
            Style::Simple => Placeholder::parse_simple(ph),
            Style::Giter8 => Placeholder::parse_g8(ph),
            Style::Pathname => Placeholder::parse_dirname(ph),
        };

        // TODO: encode missing key in `params` with proper `Err`
        // TODO: return `Err` when parsing fails
        parsed.map(|ph| ph.format_with(params)).ok()
    }

    /// Process template with given `params`, and write result into `writer`.
    pub fn write<'a, W: Write>(&mut self,
                               writer: &'a mut W,
                               params: &HashMap<String, String>)
                               -> Result<&'a mut W, io::Error> {

        let chars = self.body.as_bytes().into_iter();
        let mut found_opening = false;
        let mut marker = 0;
        let mut last_written = 0;
        let mut prev = 0u8;

        for (pos, ch) in chars.enumerate() {
            if *ch == b'$' && found_opening {

                // flush out the chunk that are not placeholder
                try!(writer.write(&self.body[last_written..marker - 1].as_bytes()));

                let ph = &self.body[marker..pos];
                if let Some(value) = self.process(ph, params) {
                    try!(writer.write(&value.as_bytes()));
                } else {
                    try!(writer.write(&self.body[marker..pos].as_bytes()));
                }
                marker = pos + 1;
                last_written = pos + 1;
                found_opening = false;

            } else if *ch == b'$' && prev != b'\\' {
                marker = pos + 1;
                found_opening = true;
            }

            if pos == self.body.as_bytes().len() - 1 {
                try!(writer.write(&self.body[last_written..pos + 1].as_bytes()));
            }

            prev = *ch;
        }

        writer.flush().unwrap();

        Ok(writer)
    }
}

/// Wrapper arround map-type collection to use as resolved parameters in project generation.
#[derive(Debug, Clone)]
pub struct Params {
    pub param_map: HashMap<String, String>
}

impl Params {

    pub fn from_map(map: HashMap<String, String>) -> Params {
        Params { param_map: map }
    }

    pub fn convert_toml(toml: &Table) -> Params {
        let mut raw_values = HashMap::new();
        for (k, tv) in toml {
            if let Some(v) = convert(tv) {
                raw_values.insert(k.clone(), v);
            }
        }
        Params { param_map: raw_values }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.param_map.get(key)
    }
}

// FIXME: should return `Result<String, errors::Error>` to tell we won't accept table / array?
fn convert(value: &Value) -> Option<String> {
    match *value {
        Value::String(_) => value.as_str().map(|s| s.to_owned()),
        Value::Datetime(_) => value.as_datetime().map(|s| s.to_owned()),
        Value::Integer(_) => value.as_integer().map(|i| i.to_string()),
        Value::Float(_) => value.as_float().map(|f| f.to_string()),
        Value::Boolean(_) => value.as_bool().map(|b| b.to_string()),
        _ => None
    }
}


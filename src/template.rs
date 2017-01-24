use std::collections::HashMap;
use std::convert::From;
use std::io::{self, Write};

use super::format::{Placeholder, Style};

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
            body: String::from(template.as_ref())
        }
    }

    /// Utility to create giter8 style template instantly.
    pub fn new_g8<S: AsRef<str>>(template: S) -> Template {
        Template::read_str(Style::Giter8, template)
    }

    /// Create template from given `str`, and instantly compile it.
    pub fn compile_inline<'a, S, W>(writer: &'a mut W,
                                    template: S,
                                    context: HashMap<String, String>)
                                    -> Result<&'a mut W, io::Error>
        where S: AsRef<str>,
              W: Write
    {
        let mut template = Template::read_str(Style::Simple, template);
        Template::write(&mut template, writer, context)
    }

    /// Replace all placeholders its holding with values from given context.
    fn process(&self, ph: &str, context: &HashMap<String, String>) -> Option<String> {

        let parsed = match self.style {
            Style::Simple => Placeholder::parse_simple(ph),
            Style::Giter8 => Placeholder::parse_g8(ph),
            Style::Pathname => Placeholder::parse_dirname(ph),
        };

        // TODO: encode missing key in `context` with proper `Err`
        // TODO: return `Err` when parsing fails
        parsed.map(|ph| ph.format_with(context)).ok()
    }

    /// Process template with given `context`, and write result into `writer`.
    pub fn write<'a, W: Write>(&mut self,
                               writer: &'a mut W,
                               context: HashMap<String, String>)
                               -> Result<&'a mut W, io::Error> {

        let chars = self.body.as_bytes().into_iter();
        let mut found_opening = false;
        let mut marker = 0;
        let mut last_written = 0;

        for (pos, ch) in chars.enumerate() {
            if *ch == b'$' && found_opening {

                // flush out the chunk that are not placeholder
                try!(writer.write(&self.body[last_written..marker - 1].as_bytes()));

                let ph = &self.body[marker..pos];
                if let Some(value) = self.process(ph, &context) {
                    try!(writer.write(&value.as_bytes()));
                } else {
                    try!(writer.write(&self.body[marker..pos].as_bytes()));
                }
                marker = pos + 1;
                last_written = pos + 1;
                found_opening = false;

            } else if *ch == b'$' {
                marker = pos + 1;
                found_opening = true;
            }

            if pos == self.body.as_bytes().len() - 1 {
                try!(writer.write(&self.body[last_written..pos + 1].as_bytes()));
            }
        }

        writer.flush().unwrap();

        Ok(writer)
    }
}

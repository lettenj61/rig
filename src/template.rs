use std::collections::HashMap;
use std::convert::From;
use std::io::{self, Write};

use super::format::{Format, format};

/// Minimal template for any kind of plain text.
#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub body: String,
}

impl Template {

    /// Create `Template` object from given `str`.
    pub fn read_str<S: AsRef<str>>(template: S) -> Template {
        Template { body: String::from(template.as_ref()) }
    }

    /// Create template from given `str`, and instantly compile it.
    pub fn compile_inline<'a, S, W>(template: S,
                                    writer: &'a mut W,
                                    context: HashMap<String, String>)
                                    -> Result<&'a mut W, io::Error>
        where S: AsRef<str>,
              W: Write
    {
        let mut template = Template::read_str(template);
        Template::write(&mut template, writer, context)
    }

    /// Replace all placeholders its holding with values from given context.
    fn process(&self, ph: &str, context: &HashMap<String, String>) -> Option<String> {

        let args = ph.split(';').collect::<Vec<_>>();

        match args.len() {
            1 => context.get(args[0]).cloned(), // TODO: encode `None` with proper `Err`.
            2 => {
                if let Some(v) = context.get(args[0]) {
                    let mut ret = v.clone();
                    let fmt_args = args[1].split(',').skip(1).collect::<Vec<_>>();
                    for f in fmt_args {
                        ret = format(&ret, f.into());
                    }
                    Some(ret)
                } else {
                    None // TODO: shuld return `Err` when expecting keys not found.
                }
            },
            _ => None // TODO: notice when parsing / formatting error occurs!
        }
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

                let placeholder = &self.body[marker..pos];
                if let Some(value) = self.process(placeholder, &context) {
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

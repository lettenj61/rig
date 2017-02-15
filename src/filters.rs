use std::collections::HashMap;

use serde_json::value::{Value, to_value};
use tera::{ErrorKind, Result};

use super::format::{format, Formatter};

macro_rules! convert_tera_filter {
    ( $($fn_name:ident, $name:expr, $fv:ident);+ ) => {
        $(
            pub fn $fn_name(value: Value, _: HashMap<String, Value>) -> Result<Value> {
                let s = try_get_value!($name, "value", String, value);
                to_value(format(&s, Formatter::$fv)).map_err(|e| ErrorKind::Json(e).into())
            }
        )*
    };
}

convert_tera_filter! {
    decap, "decap", Decapitalize;
    word, "word", WordChar;
    hyphen, "hyphen", Hyphenate;
    start, "start", StartCase;
    upper_camel, "Camel", UpperCamel;
    lower_camel, "camel", LowerCamel;
    norm, "norm", Normalize;
    snake, "snake", SnakeCase;
    packaged, "packaged", DirectoryPath;
    random, "random", AddRandom
}
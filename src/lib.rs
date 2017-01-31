#![recursion_limit = "1024"]

extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate git2;
extern crate java_properties;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate tempdir;
extern crate toml;
extern crate url;
extern crate walkdir;

pub mod errors;
pub mod format;
pub mod fsutils;
pub mod project;
pub mod template;

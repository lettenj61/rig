use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fmt;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use java_properties;
use tempdir::TempDir;
use toml;
use url::Url;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use super::errors::*;
use super::format::{self, Placeholder, Style};
use super::fsutils;
use super::template::Template;

type Context = HashMap<String, String>;

#[derive(Debug)]
pub struct Project {
    pub root_path: Option<String>,
    pub config_format: ConfigFormat,
    pub style: Style,
}

#[derive(Copy, Clone, Debug)]
pub enum ConfigFormat {
    JavaProps,
    Toml,
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        let name = match *self {
            ConfigFormat::JavaProps => "properties",
            ConfigFormat::Toml => "toml",
        };
        write!(f, "default.{}", &name)
    }
}

impl Default for Project {
    fn default() -> Project {
        Project {
            root_path: None,
            config_format: ConfigFormat::Toml,
            style: Style::Simple,
        }
    }
}

impl Project {
    pub fn new_g8(root: Option<&str>) -> Project {
        Project {
            root_path: root.map(|v| v.to_string()),
            config_format: ConfigFormat::JavaProps,
            style: Style::Giter8,
        }
    }

    pub fn set_root_dir(&mut self, root: &str) -> &mut Project {
        self.root_path = Some(root.into());
        self
    }

    pub fn resolve_root_dir(&self, clone_root: &Path) -> PathBuf {
        let mut buf = clone_root.to_path_buf();

        if let Some(ref inner) = self.root_path {
            if fsutils::exists(clone_root.join(inner)) {
                buf.push(inner);
            }
        }
        buf
    }

    pub fn default_context(&self, clone_root: &Path) -> Result<Context> {
        let root = self.resolve_root_dir(clone_root);
        get_default_context(self, &root)
    }

    // TODO: give clear `Err` type
    // TODO: make it run async
    pub fn generate(&self,
                    context: &Context,
                    clone_root: &Path,
                    dest: &Path,
                    dry_run: bool)
                    -> Result<()> {

        let root = self.resolve_root_dir(clone_root);
        let walker = WalkDir::new(&root).into_iter();

        let mut file_map: HashMap<String, String> = HashMap::new();
        let default_ctx = clone_root.join(format!("{}", &self.config_format));

        if !dry_run {
            fs::create_dir_all(dest).unwrap();
        }

        for entry in walker.filter_entry(|e| !is_git_metadata(e)) {
            let entry = entry.unwrap();

            if entry.path() == &root || entry.path() == &default_ctx {
                debug!("skipping {:?}", entry.file_name());
                continue;
            }

            let mut segment: Vec<&OsStr> = Vec::new();
            let mut rel_path_up = entry.path().parent();
            let mut upwards = 1;
            while let Some(parent) = rel_path_up {
                if upwards >= entry.depth() {
                    break;
                } else {
                    segment.push(parent.file_name().unwrap_or("".as_ref()));
                }
                upwards += 1;
                rel_path_up = parent.parent();
            }

            let from = entry.path().to_string_lossy().to_string();
            let mut dest = dest.to_path_buf();
            if !segment.is_empty() {
                segment.reverse();
                for part in segment {
                    if let Some(rep) = file_map.get(&from) {
                        dest.push(rep);
                    } else {
                        dest.push(part);
                    }
                }
            }

            let mut name = entry.file_name().to_string_lossy().to_string();
            if format::is_placeholder(&name) {
                let ph = Placeholder::parse_dirname(&name).unwrap();
                name = ph.format_with(&context);
                file_map.insert(from.clone(), name.clone());
            }
            dest.push(&name);
            debug!("Destination entry: {:?}", dest);

            // TODO:
            if !dry_run {
                if entry.file_type().is_file() {
                    let mut tpl = Template::read_file(self.style.clone(), &entry.path()).unwrap();
                    let mut f = fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(dest.as_path())
                        .unwrap();

                    tpl.write(&mut f, context).unwrap();
                    f.sync_data().unwrap();
                    // fs::copy(&entry.path(), dest.as_path()).expect("Failed to copy file");
                } else if entry.file_type().is_dir() {
                    fs::create_dir_all(dest.as_path()).expect("Failed to copy directory");
                }
            }
        }

        Ok(())
    }
}

fn is_git_metadata(entry: &DirEntry) -> bool {
    let is_git = entry.file_name().to_str().map(|s| s == ".git").unwrap_or(false);
    fsutils::is_directory(entry.path()) && is_git
}

fn get_default_context(project: &Project, root_dir: &Path) -> Result<Context> {
    let default_ctx = root_dir.join(format!("{}", project.config_format));
    debug!("{:?}", default_ctx);

    match project.config_format {
        ConfigFormat::JavaProps => {
            fs::File::open(&default_ctx)
                .map(|f| java_properties::read(f).unwrap())
                .map_err(|e| ErrorKind::Io(e).into())
        }
        ConfigFormat::Toml => {
            fsutils::read_file(&default_ctx)
                .map(|s| toml::decode_str::<HashMap<String, String>>(&s).unwrap())
                .chain_err(|| ErrorKind::TomlDecodeFailure)
        }
    }
}

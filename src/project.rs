use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use java_properties;
use toml;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use super::errors::*;
use super::fsutils;
use super::template::{Placeholder, Style, Params, Template};

#[derive(Debug)]
pub struct Project {
    pub root_path: Option<String>,
    pub config_format: ConfigFormat,
    pub style: Style,
    pub force_packaged: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum ConfigFormat {
    JavaProps,
    Toml,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            root_path: None,
            config_format: ConfigFormat::Toml,
            style: Style::Tera,
            force_packaged: false,
        }
    }
}

impl Project {
    pub fn new<S>(root: Option<S>, config_format: ConfigFormat, packaged: bool) -> Project
        where S: AsRef<str>
    {
        Project {
            root_path: root.map(|v| v.as_ref().to_owned()),
            config_format: config_format,
            style: Style::Tera,
            force_packaged: packaged,
        }
    }

    pub fn new_g8(root: Option<&str>) -> Project {
        Project {
            root_path: root.map(|v| v.to_string()),
            config_format: ConfigFormat::JavaProps,
            style: Style::ST,
            force_packaged: true,
        }
    }

    pub fn config_name(&self) -> &'static str {
        match self.config_format {
            ConfigFormat::JavaProps => "default.properties",
            ConfigFormat::Toml => "_rig.toml",
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

    pub fn default_params(&self, clone_root: &Path) -> Result<Params> {
        let root = self.resolve_root_dir(clone_root);
        get_defaults(self, &root)
    }

    // TODO: give clear `Err` type
    // TODO: make it run async
    pub fn generate(&self,
                    params: &Params,
                    clone_root: &Path,
                    dest: &Path,
                    dry_run: bool)
                    -> Result<()> {

        let root = self.resolve_root_dir(clone_root);
        let walker = WalkDir::new(&root).into_iter();

        let mut file_map: HashMap<OsString, String> = HashMap::new();
        let default_file = root.join(self.config_name());

        if !dry_run {
            fs::create_dir_all(dest).unwrap();
        }

        for entry in walker.filter_entry(|e| !is_git_metadata(e)) {
            let entry = entry.unwrap();

            if entry.path() == &root || entry.path() == &default_file {
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

            let base = entry.file_name();
            let mut dest = dest.to_path_buf();
            if !segment.is_empty() {
                segment.reverse();
                for part in segment {
                    if let Some(rep) = file_map.get(part) {
                        debug!("File tree altered: {:?} => {:?}", part, rep);
                        dest.push(rep);
                    } else {
                        dest.push(part);
                    }
                }
            }

            let mut buf = Vec::new();
            // FIXME: we need to re-design `Template` so we can manipulate its elements
            if "$package$" == base.to_string_lossy().as_ref() && self.force_packaged {
                Template::write_once(&mut buf,
                                     Style::Path,
                                     "$package__packaged$",
                                     &params.param_map)
                    .unwrap();
            } else {
                Template::write_once(&mut buf,
                                     Style::Path,
                                     &base.to_string_lossy(),
                                     &params.param_map)
                    .unwrap();
            }

            let name = String::from_utf8(buf).unwrap();
            if &name != base.to_string_lossy().as_ref() {
                file_map.insert(base.to_os_string(), name.clone());
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

                    tpl.write_to(&mut f, &params.param_map).unwrap();
                    f.sync_data().unwrap();
                } else if entry.file_type().is_dir() {
                    fs::create_dir_all(dest.as_path()).expect("Failed to copy directory");
                }
            }
        }
        debug!("{:?}", &file_map);

        Ok(())
    }
}

fn is_git_metadata(entry: &DirEntry) -> bool {
    let is_git = entry.file_name().to_str().map(|s| s == ".git").unwrap_or(false);
    fsutils::is_directory(entry.path()) && is_git
}

fn get_defaults(project: &Project, root_dir: &Path) -> Result<Params> {
    let defaults_file = root_dir.join(project.config_name());

    match project.config_format {
        ConfigFormat::JavaProps => {
            fs::File::open(&defaults_file)
                .map(|f| {
                    let props = java_properties::read(f).unwrap();
                    Params::from_map(props)
                })
                .map_err(|e| ErrorKind::Io(e).into()) // Should convert ParseError
        }
        ConfigFormat::Toml => {
            fsutils::read_file(&defaults_file)
                .map(|s| {
                    let mut parser = toml::Parser::new(&s);
                    let tbl = parser.parse().unwrap();
                    Params::convert_toml(&tbl)
                })
                .chain_err(|| ErrorKind::TomlDecodeFailure)
        }
    }
}

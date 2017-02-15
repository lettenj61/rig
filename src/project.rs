use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use java_properties;
use tera::{Context, Tera};
use toml;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

use super::errors::*;
use super::filters;
use super::fsutils;
use super::template::{Style, Params, Template};

#[derive(Debug)]
pub struct Project {
    pub root_path: Option<String>,
    pub config: Configuration,
    pub style: Style,
    pub force_packaged: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Configuration {
    JavaProps,
    Toml,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            root_path: None,
            config: Configuration::Toml,
            style: Style::Tera,
            force_packaged: false,
        }
    }
}

impl Project {
    pub fn new<S>(root: Option<S>, config: Configuration, packaged: bool) -> Project
        where S: AsRef<str>
    {
        Project {
            root_path: root.map(|v| v.as_ref().to_owned()),
            config: config,
            style: Style::Tera,
            force_packaged: packaged,
        }
    }

    pub fn new_g8(root: Option<&str>) -> Project {
        Project {
            root_path: root.map(|v| v.to_string()),
            config: Configuration::JavaProps,
            style: Style::ST,
            force_packaged: true,
        }
    }

    pub fn config_name(&self) -> &'static str {
        match self.config {
            Configuration::JavaProps => "default.properties",
            Configuration::Toml => "Rig.toml",
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

        let mut name_map: HashMap<OsString, String> = HashMap::new();
        let mut tree: Vec<(DirEntry, PathBuf)> = Vec::new();
        let default_file = root.join(self.config_name());

        for entry in walker.filter_entry(|e| !is_git_metadata(e)) {
            let entry = entry.unwrap();

            if entry.path() == &root || entry.path() == &default_file {
                debug!("skipping {:?}", entry.file_name());
                continue;
            }

            &tree.push((entry.clone(), resolve_dirname(self, &entry, dest, &mut name_map, params)));

        }
        // TODO:
        if !dry_run {
            fs::create_dir_all(dest).unwrap();
            match self.style {
                Style::Tera => self.generate_with_tera(params, tree),
                _ => self.generate_tree(params, tree)
            }
        }
        debug!("{:?}", &name_map);

        Ok(())
    }

    fn generate_tree(&self, params: &Params, tree: Vec<(DirEntry, PathBuf)>) {

        for loc in tree {
            let (src, dest) = loc;

            if src.file_type().is_file() {

                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(dest.as_path())
                    .unwrap();

                let mut tpl = Template::read_file(self.style.clone(),
                                                  &src.path())
                    .unwrap();
                tpl.write_to(&mut f, &params.param_map).unwrap();
                f.sync_data().unwrap();

            } else if src.file_type().is_dir() {
                fs::create_dir_all(dest.as_path()).expect("Creating directory");
            }
        }
    }

    fn generate_with_tera(&self,
                          params: &Params,
                          tree: Vec<(DirEntry, PathBuf)>) {

        let mut tera = Tera::default();
        let mut ctx = Context::new();
        init_tera_filters(&mut tera);

        // TODO: which toml table will be used in context?
        for (k, v) in &params.param_map {
            &ctx.add(&k, &v);
        }

        for ref loc in &tree {
            let (ref src, ref dest) = **loc;
            if src.file_type().is_file() {
                tera.add_template_file(&src.path(),
                                       Some(dest.to_string_lossy().as_ref()))
                    .unwrap();
            }
        }
        debug!("{:?}", &tera.templates);

        for loc in tree {
            let (src, dest) = loc;
            debug!("{:?} => {:?}", &src, &dest);

            if src.file_type().is_file() {

                let content = tera
                    .render(dest.to_string_lossy().as_ref(), ctx.clone())
                    .unwrap();

                fsutils::write_file(&dest, &content).unwrap();
            } else {
                fs::create_dir_all(dest.as_path()).expect("Creating directory");
            }
        }
    }
}

fn is_git_metadata(entry: &DirEntry) -> bool {
    let is_git = entry.file_name().to_str().map(|s| s == ".git").unwrap_or(false);
    fsutils::is_directory(entry.path()) && is_git
}

fn resolve_dirname(project: &Project,
                   entry: &DirEntry,
                   dest_root: &Path,
                   alt_paths: &mut HashMap<OsString, String>,
                   params: &Params)
                   -> PathBuf
{

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
    let mut dest = dest_root.to_path_buf();
    if !segment.is_empty() {
        segment.reverse();
        for part in segment {
            if let Some(rep) = alt_paths.get(part) {
                debug!("File tree altered: {:?} => {:?}", part, rep);
                dest.push(rep);
            } else {
                dest.push(part);
            }
        }
    }

    let mut buf = Vec::new();
    // FIXME: we need to re-design `Template` so we can manipulate its elements
    let mut pkg = base.to_string_lossy();
    if pkg.as_ref() == "$package$" && project.force_packaged {
        pkg = Cow::from("$package__packaged$");
    }
    Template::write_once(&mut buf,
                         Style::Path,
                         pkg,
                         &params.param_map)
        .unwrap();

    let name = String::from_utf8(buf).unwrap();
    if &name != base.to_string_lossy().as_ref() {
        alt_paths.insert(base.to_os_string(), name.clone());
    }
    dest.push(&name);
    debug!("Destination entry: {:?}", dest);

    dest
}

fn get_defaults(project: &Project, root_dir: &Path) -> Result<Params> {
    let defaults_file = root_dir.join(project.config_name());

    // TODO: get default value from specific toml table if there is any
    match project.config {
        Configuration::JavaProps => {
            fs::File::open(&defaults_file)
                .map(|f| {
                    let props = java_properties::read(f).unwrap();
                    Params::from_map(props)
                })
                .map_err(|e| ErrorKind::Io(e).into()) // Should convert ParseError
        }
        Configuration::Toml => {
            fsutils::read_file(&defaults_file)
                .map(|s| {
                    let tbl: toml::value::Table = toml::from_str(&s).unwrap();
                    Params::convert_toml(tbl)
                })
                .chain_err(|| ErrorKind::TomlDecodeFailure)
        }
    }
}

fn init_tera_filters(tera: &mut Tera) {
    tera.register_filter("decap", filters::decap);
    tera.register_filter("word", filters::word);
    tera.register_filter("hyphen", filters::hyphen);
    tera.register_filter("start", filters::start);
    tera.register_filter("Camel", filters::upper_camel);
    tera.register_filter("camel", filters::lower_camel);
    tera.register_filter("norm", filters::norm);
    tera.register_filter("snake", filters::snake);
    tera.register_filter("packaged", filters::packaged);
    tera.register_filter("random", filters::random);
}
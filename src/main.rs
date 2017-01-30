extern crate env_logger;
extern crate git2;
extern crate java_properties;
#[macro_use]
extern crate log;
extern crate tempdir;
extern crate url;
extern crate walkdir;

extern crate rig;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use git2::{Config as Git2Config, FetchOptions};
use git2::build::RepoBuilder;
use rig::fsutils;
use rig::project::{ConfigFormat, Project};
use tempdir::TempDir;
use url::Url;
use walkdir::{DirEntry, WalkDir, WalkDirIterator};

fn find_proxy_url() -> Option<Url> {

    // environment variables are first priority
    if let Some(env_val) = env::var_os("http_proxy") {
        debug!("Setting proxy configuration from environment key: `http_proxy`.");
        Url::parse(&env_val.to_string_lossy()).ok()
    } else {
        // if no env vars set, look for git global config
        if let Ok(global_conf) = Git2Config::find_global() {

            if let Ok(config) = Git2Config::open(global_conf.as_path()) {
                config.get_string("http.proxy").map(|v| Url::parse(&v).unwrap()).ok()
            } else {
                warn!("Cannot locate or open git global configuration");
                None
            }

        } else {
            None
        }
    }
}

fn is_git_metadata(entry: &DirEntry) -> bool {
    let is_git = entry.file_name().to_str().map(|s| s == ".git").unwrap_or(false);
    fsutils::is_directory(entry.path()) && is_git
}

fn ensure_inner_project(path: &Path) -> PathBuf {
    let mut buf = path.to_path_buf();

    if fsutils::exists(path.join("src/main/g8")) {
        buf.push("src/main/g8");

        debug!("Found inner project at: {:?}", buf);
    }

    buf
}

fn main() {

    env_logger::init().unwrap();
    let dry_run = true;

    let clone_root = TempDir::new("rig__template").expect("Failed to create temporal directory!");

    let mut output_dir = clone_root.path()
        .parent()
        .map(|p| p.to_path_buf())
        .expect("Failed to locate current directory!");
    output_dir.push("_test_out");

    if fsutils::exists(&output_dir) {
        fsutils::remove_dir(&output_dir).expect("Cannot overwrite existing output directory!");
    }
    fs::create_dir_all(&output_dir).unwrap();

    let url = Url::parse("https://github.com/n8han/giter8.g8").unwrap();
    let mut _in = String::new();

    let mut repo = RepoBuilder::new();
    if let Some(proxy_url) = find_proxy_url() {

        debug!("Proxy settings found, initializing fetch options.");

        let mut proxy = git2::ProxyOptions::new();
        proxy.url(proxy_url.as_ref());

        let mut fetch = FetchOptions::new();
        fetch.proxy_options(proxy);

        repo.fetch_options(fetch);
    } else {
        debug!("No proxy settings found.")
    }

    info!("Cloning remote git repository: {:?} into {:?}",
          url,
          clone_root.path());
    let _ = repo.clone(url.as_ref(), &clone_root.path()).unwrap();

    let mut project = Project::new_g8(None);
    let mut template_root = ensure_inner_project(&clone_root.path());

    let settings = template_root.join(format!("{}", project.config_format));
    if let Ok(key_value) = fs::File::open(&settings).map(java_properties::read) {
        println!("{:?}", key_value);
    }

    let walker = WalkDir::new(&template_root).into_iter();
    for entry in walker.filter_entry(|e| !is_git_metadata(e)) {

        let entry = entry.unwrap();

        if entry.path() == &template_root {
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

        let mut dest = output_dir.clone();
        if !segment.is_empty() {
            segment.reverse();
            for part in segment {
                dest.push(part);
            }
        }
        dest.push(entry.file_name());
        debug!("{:?}", dest);

        if !dry_run {
            if entry.file_type().is_file() {
                fs::copy(&entry.path(), dest.as_path()).expect("Failed to copy file");
            } else if entry.file_type().is_dir() {
                fs::create_dir(dest.as_path()).expect("Failed to copy directory");
            }
        }
    }

    info!("done.");
}

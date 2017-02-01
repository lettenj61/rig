extern crate docopt;
extern crate error_chain;
extern crate env_logger;
extern crate git2;
extern crate java_properties;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate tempdir;
extern crate url;

extern crate rig;

use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use docopt::Docopt;
use git2::{Config as Git2Config, FetchOptions};
use git2::build::RepoBuilder;
use tempdir::TempDir;
use url::Url;

use rig::errors::*;
use rig::format::{format, Format};
use rig::project::{ConfigFormat, Project};

const USAGE: &'static str = r#"
Rig - Generate new project by cloning templates from git repository.

*NOTE* This software is under early development, most of its features are not yet supported:
  - Currently it can only use templates that hosted on GitHub
  - giter8 compatibility features (e.g. maven directive) are not yet supported.

Usage:
    rig <repository> [options]
    rig (-h | --help)
    rig (-V | --version)

Options:
    -h, --help              Show help message
    -V, --version           Show version
    --name NAME             Specify project name (overrides default if any)
    --output PATH           Specify output directory to generate project
    --root PATH             Specify directory where template lives in repository
    --verbatim EXTENSION    Space separeted list of file exts exclude from template processing
    -p, --packaged          Force format `package` parameter value into directory tree
    -Y, --confirm           Use template default value to all parameters (Yes-To-All)
    --dry-run               Show generation process to STDOUT, without producing any files
    --giter8                Expects that the template is a giter8 template
    --no-logo               Supress logo
"#;

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_repository: String,
    flag_name: Option<String>,
    flag_output: Option<String>,
    flag_root: Option<String>,
    flag_verbatim: Option<String>, // unimplemented!
    flag_packaged: bool,
    flag_confirm: bool,
    flag_giter8: bool,
    flag_dry_run: bool,
    flag_no_logo: bool, // I wish someday I could draw some logo
    flag_help: bool,
    flag_version: bool,
}

fn main() {

    env_logger::init().unwrap();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    debug!("{:?}", args);

    if args.flag_version {
        println!("Rig - 0.1.0");
        exit(0);
    }

    // gather info of remote repository & networks
    let url = normalize_url(&args.arg_repository).unwrap();
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

    let clone_root = TempDir::new("rig__template").expect("Failed to create temporal directory");
    info!("Cloning remote git repository: {:?} into {:?}",
          url,
          clone_root.path());
    let _ = repo.clone(url.as_ref(), &clone_root.path()).unwrap();

    let project = match args.flag_giter8 {
        true => Project::new_g8(Some("src/main/g8")),
        false => Project::new(args.flag_root.as_ref(),
                              ConfigFormat::Toml, // TODO: parameterize config format
                              args.flag_packaged),
    };

    let mut context = project.default_context(&clone_root.path()).unwrap();
    debug!("Successfully read default context: {:?}", context);

    if !args.flag_confirm {
        collect_params(&args.flag_name, &mut context);
        debug!("Context updated with user input: {:?}", context);
    }

    // ensure we have real path to output directory
    let project_name = context.get("name")
        .cloned()
        .unwrap_or("Rig Generated Project".to_owned());

    let output_dir = get_output_dir(&args.flag_output, &project_name);
    debug!("Set output directory: {:?}", output_dir);

    project.generate(&context, &clone_root.path(), &output_dir, args.flag_dry_run).unwrap();

    println!("Project successfully generated: {:?}", &output_dir);
    drop(clone_root);
}

fn find_proxy_url() -> Option<Url> {

    // we take env vars first
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

fn normalize_url(raw: &str) -> Result<Url> {
    if let Some(_) = raw.find('/') {
        Url::parse(raw)
            .or(Url::parse(format!("https://github.com/{}", raw).as_ref()))
            .map_err(|e| ErrorKind::ParseUrl(e).into())
    } else {
        Err(ErrorKind::InvalidUrlFormat(raw.to_string()).into())
    }
}

fn collect_params<'a>(name: &'a Option<String>,
                      context: &'a mut HashMap<String, String>)
                      -> &'a mut HashMap<String, String> {
    let mut s = String::new();
    for (k, v) in context.iter_mut() {

        // we treat `name` parameter specially
        if k == "name" {
            if let Some(ref arg_name) = *name {
                *v = arg_name.clone();
                continue;
            }
        }

        print!("{} [{}]:", k, v);
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut s).unwrap();
        if !s.trim().is_empty() {
            *v = s.trim().to_string();
            s.clear();
        }
    }
    context
}

fn get_output_dir(arg_name: &Option<String>, default_name: &str) -> PathBuf {
    let mut output_dir = env::current_dir().unwrap();
    if let Some(ref name) = *arg_name {
        let path = Path::new(name);
        if path.is_relative() {
            let normalized = format(name, Format::Normalize);
            output_dir.push(&normalized);
        } else if path.is_absolute() {
            output_dir = path.to_path_buf();
        }
    } else {
        output_dir.push(&format(default_name, Format::Normalize));
    }

    output_dir
}

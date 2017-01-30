use std::path::Path;

#[derive(Debug)]
pub struct Project {
    pub root_dir: String,
    pub altered_root: Option<String>,
    pub config_format: ConfigFormat,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ConfigFormat {
    JavaProps,
    Toml,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            root_dir: ".".into(),
            altered_root: None,
            config_format: ConfigFormat::Toml
        }
    }
}

impl Project {

    pub fn alter_root(&mut self, root: &str) -> &mut Project {
        self.altered_root = Some(root.into());
        self
    }
}
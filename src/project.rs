use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub struct Project {
    pub altered_root: Option<String>,
    pub config_format: ConfigFormat,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ConfigFormat {
    JavaProps,
    Toml,
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let name = match *self {
            ConfigFormat::JavaProps => "default.properties",
            ConfigFormat::Toml => "default.toml"
        };
        write!(f, "{}", &name)
    }
}

impl Default for Project {
    fn default() -> Project {
        Project {
            altered_root: None,
            config_format: ConfigFormat::Toml
        }
    }
}

impl Project {

    pub fn new_g8(root: Option<&str>) -> Project {
        Project {
            altered_root: root.map(|v| v.to_string()),
            config_format: ConfigFormat::JavaProps,
        }
    }

    pub fn alter_root(&mut self, root: &str) -> &mut Project {
        self.altered_root = Some(root.into());
        self
    }
}
use std::path::Path;

#[derive(Debug)]
pub struct Project {
    root_dir: String,
    template_dir: Option<String>,
    config_format: ConfigFormat,
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
            template_dir: None,
            config_format: ConfigFormat::Toml
        }
    }
}

impl Project {

    pub fn alter_root(&mut self, root: &str) -> &mut Project {
        self.template_dir = Some(root.into());
        self
    }
}
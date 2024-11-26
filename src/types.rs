use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    pub resource_type: String,
    pub name: String,
    pub is_module: bool,
    pub file_path: PathBuf,
}

impl Resource {
    pub fn full_name(&self) -> String {
        if self.is_module {
            format!("module.{}", self.name)
        } else {
            format!("{}.{}", self.resource_type, self.name)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    File(PathBuf),
    Module(String),
    Resource(String, String),
}

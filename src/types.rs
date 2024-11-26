use std::path::PathBuf;

/// Represents a Terraform resource with extended metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// The type of the resource (e.g., "aws_instance", "local_file")
    pub resource_type: String,
    /// The name of the resource
    pub name: String,
    /// Whether this is a module
    pub is_module: bool,
    /// Path to the file containing this resource
    pub file_path: PathBuf,
    /// Whether the resource uses count
    pub has_count: bool,
    /// Whether the resource uses for_each
    pub has_for_each: bool,
    /// The specific index for count/for_each resources
    pub index: Option<String>,
}

impl Resource {
    /// Returns the full name of the resource in Terraform format
    pub fn full_name(&self) -> String {
        if self.is_module {
            format!("module.{}", self.name)
        } else {
            format!("{}.{}", self.resource_type, self.name)
        }
    }

    /// Returns the target string for Terraform commands
    pub fn target_string(&self) -> String {
        let base = self.full_name();
        match (&self.has_count, &self.has_for_each, &self.index) {
            (true, _, Some(idx)) | (_, true, Some(idx)) => format!("{}[{}]", base, idx),
            _ => base,
        }
    }
}

/// Represents different types of targets for Terraform operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    File(PathBuf),
    Module(String),
    Resource(String, String),
}

use log::debug;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TfocusError};
use crate::types::{Resource, Target};

/// Represents a Terraform project with its resources
pub struct TerraformProject {
    resources: Vec<Resource>,
}

impl TerraformProject {
    /// Creates a new empty TerraformProject
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Recursively finds all Terraform files in the given directory
    fn find_terraform_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut tf_files = Vec::new();

        for entry in fs::read_dir(dir).map_err(TfocusError::Io)? {
            let entry = entry.map_err(TfocusError::Io)?;
            let path = entry.path();

            if path.is_file() {
                if path.extension().map_or(false, |ext| ext == "tf")
                    && !path.to_string_lossy().contains("/.terraform/")
                {
                    tf_files.push(path);
                }
            } else if path.is_dir()
                && !path.to_string_lossy().contains("/.terraform/")
                && !path.to_string_lossy().contains("/.git/")
            {
                tf_files.extend(Self::find_terraform_files(&path)?);
            }
        }

        Ok(tf_files)
    }

    /// Parses a directory containing Terraform files
    pub fn parse_directory(path: &Path) -> Result<Self> {
        let mut project = TerraformProject::new();

        let tf_files = Self::find_terraform_files(path)?;
        if tf_files.is_empty() {
            return Err(TfocusError::NoTerraformFiles);
        }

        println!("\nFound Terraform files:");
        for file in &tf_files {
            if let Ok(rel_path) = file.strip_prefix(path) {
                println!("  {}", rel_path.display());
            } else {
                println!("  {}", file.display());
            }
        }
        println!();

        for file_path in tf_files {
            project.parse_file(&file_path)?;
        }

        Ok(project)
    }

    /// Parses a single Terraform file for resources and modules
    fn parse_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(TfocusError::Io)?;
        debug!("Parsing file: {:?}", path);

        // Parse resources with improved regex pattern
        let resource_regex =
            Regex::new(r#"(?m)^\s*resource\s+"([^"]+)"\s+"([^"]+)"\s*\{(?s:.*?)\n\s*\}"#)
                .map_err(TfocusError::RegexError)?;

        for cap in resource_regex.captures_iter(&content) {
            let full_block = cap.get(0).unwrap().as_str();
            let has_count = full_block.contains("count =") || full_block.contains("count=");
            let has_for_each =
                full_block.contains("for_each =") || full_block.contains("for_each=");

            self.resources.push(Resource {
                resource_type: cap[1].to_string(),
                name: cap[2].to_string(),
                is_module: false,
                file_path: path.to_owned(),
                has_count,
                has_for_each,
                index: None,
            });
        }

        // Parse modules with improved regex pattern
        let module_regex = Regex::new(r#"(?m)^\s*module\s+"([^"]+)"\s*\{(?s:.*?)\n\s*\}"#)
            .map_err(TfocusError::RegexError)?;

        for cap in module_regex.captures_iter(&content) {
            let full_block = cap.get(0).unwrap().as_str();
            let has_count = full_block.contains("count =") || full_block.contains("count=");
            let has_for_each =
                full_block.contains("for_each =") || full_block.contains("for_each=");

            self.resources.push(Resource {
                resource_type: String::new(),
                name: cap[1].to_string(),
                is_module: true,
                file_path: path.to_owned(),
                has_count,
                has_for_each,
                index: None,
            });
        }

        Ok(())
    }

    /// Returns a list of unique file paths
    pub fn get_unique_files(&self) -> Vec<PathBuf> {
        let mut files: HashSet<PathBuf> = HashSet::new();
        for resource in &self.resources {
            files.insert(resource.file_path.clone());
        }
        let mut files: Vec<_> = files.into_iter().collect();
        files.sort();
        files
    }

    /// Returns a list of module names
    pub fn get_modules(&self) -> Vec<String> {
        let mut modules: Vec<String> = self
            .resources
            .iter()
            .filter(|r| r.is_module)
            .map(|r| r.name.clone())
            .collect();
        modules.sort();
        modules.dedup();
        modules
    }

    /// Returns all resources in the project
    pub fn get_all_resources(&self) -> Vec<Resource> {
        let mut resources = self.resources.clone();
        resources.sort_by(|a, b| {
            if a.is_module == b.is_module {
                a.full_name().cmp(&b.full_name())
            } else {
                b.is_module.cmp(&a.is_module)
            }
        });
        resources
    }

    /// Returns resources matching the specified target
    pub fn get_resources_by_target(&self, target: &Target) -> Vec<Resource> {
        match target {
            Target::File(path) => self
                .resources
                .iter()
                .filter(|r| &r.file_path == path)
                .cloned()
                .collect(),
            Target::Module(module_name) => self
                .resources
                .iter()
                .filter(|r| r.is_module && &r.name == module_name)
                .cloned()
                .collect(),
            Target::Resource(resource_type, name) => self
                .resources
                .iter()
                .filter(|r| !r.is_module && &r.resource_type == resource_type && &r.name == name)
                .cloned()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_resource_with_count() {
        let mut project = TerraformProject::new();
        let content = r#"
        resource "aws_instance" "web" {
          count = 2
          ami = "ami-123456"
          instance_type = "t2.micro"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        project.parse_file(temp_file.path()).unwrap();

        let resources = project.get_all_resources();
        assert_eq!(resources.len(), 1, "Expected exactly one resource");
        assert!(resources[0].has_count, "Resource should have count");
        assert!(
            !resources[0].has_for_each,
            "Resource should not have for_each"
        );
    }

    #[test]
    fn test_parse_resource_with_for_each() {
        let mut project = TerraformProject::new();
        let content = r#"
        resource "aws_instance" "web" {
          for_each = toset(["a", "b"])
          ami = "ami-123456"
          instance_type = "t2.micro"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        project.parse_file(temp_file.path()).unwrap();

        let resources = project.get_all_resources();
        assert_eq!(resources.len(), 1, "Expected exactly one resource");
        assert!(!resources[0].has_count, "Resource should not have count");
        assert!(resources[0].has_for_each, "Resource should have for_each");
    }

    #[test]
    fn test_parse_module_with_count() {
        let mut project = TerraformProject::new();
        let content = r#"
        module "web" {
          count = 2
          source = "./modules/web"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        project.parse_file(temp_file.path()).unwrap();

        let resources = project.get_all_resources();
        assert_eq!(resources.len(), 1, "Expected exactly one module");
        assert!(resources[0].has_count, "Module should have count");
        assert!(
            !resources[0].has_for_each,
            "Module should not have for_each"
        );
        assert!(resources[0].is_module, "Resource should be a module");
    }

    #[test]
    fn test_parse_module_with_for_each() {
        let mut project = TerraformProject::new();
        let content = r#"
        module "web" {
          for_each = toset(["a", "b"])
          source = "./modules/web"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        project.parse_file(temp_file.path()).unwrap();

        let resources = project.get_all_resources();
        assert_eq!(resources.len(), 1, "Expected exactly one module");
        assert!(!resources[0].has_count, "Module should not have count");
        assert!(resources[0].has_for_each, "Module should have for_each");
        assert!(resources[0].is_module, "Resource should be a module");
    }

    #[test]
    fn test_get_resources_by_target() {
        let mut project = TerraformProject::new();
        let content = r#"
        resource "aws_instance" "web" {
          count = 2
          ami = "ami-123456"
          instance_type = "t2.micro"
        }

        module "app" {
          source = "./modules/app"
        }
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();
        let file_path = temp_file.path().to_path_buf();

        project.parse_file(&file_path).unwrap();

        let by_file = project.get_resources_by_target(&Target::File(file_path.clone()));
        assert_eq!(by_file.len(), 2, "Expected two resources in the file");

        let by_resource = project.get_resources_by_target(&Target::Resource(
            "aws_instance".to_string(),
            "web".to_string(),
        ));
        assert_eq!(by_resource.len(), 1, "Expected one matching resource");
        assert!(by_resource[0].has_count, "Resource should have count");

        let by_module = project.get_resources_by_target(&Target::Module("app".to_string()));
        assert_eq!(by_module.len(), 1, "Expected one matching module");
        assert!(by_module[0].is_module, "Resource should be a module");
    }
}

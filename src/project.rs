use log::debug;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TfocusError};
use crate::types::{Resource, Target};

pub struct TerraformProject {
    resources: Vec<Resource>,
}

impl TerraformProject {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

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

    fn parse_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path).map_err(TfocusError::Io)?;

        debug!("Parsing file: {:?}", path);

        // Parse resources
        let resource_regex = Regex::new(r#"(?m)^resource\s+"([^"]+)"\s+"([^"]+)""#)
            .map_err(TfocusError::RegexError)?;
        for cap in resource_regex.captures_iter(&content) {
            self.resources.push(Resource {
                resource_type: cap[1].to_string(),
                name: cap[2].to_string(),
                is_module: false,
                file_path: path.to_owned(),
            });
        }

        // Parse modules
        let module_regex =
            Regex::new(r#"(?m)^module\s+"([^"]+)""#).map_err(TfocusError::RegexError)?;
        for cap in module_regex.captures_iter(&content) {
            self.resources.push(Resource {
                resource_type: String::new(),
                name: cap[1].to_string(),
                is_module: true,
                file_path: path.to_owned(),
            });
        }

        Ok(())
    }

    pub fn get_unique_files(&self) -> Vec<PathBuf> {
        let mut files: HashSet<PathBuf> = HashSet::new();
        for resource in &self.resources {
            files.insert(resource.file_path.clone());
        }
        let mut files: Vec<_> = files.into_iter().collect();
        files.sort();
        files
    }

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

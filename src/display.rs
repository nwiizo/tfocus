use crate::types::Resource;
use colored::*;

pub struct Display;

impl Display {
    pub fn print_header(text: &str) {
        println!("\n{}", text.bright_blue().bold());
    }

    pub fn print_resource(resource: &Resource) {
        let prefix = if resource.is_module {
            format!("[{}]", "Module".green())
        } else {
            format!("[{}]", "Resource".blue())
        };

        println!(
            "- {} {} ({})",
            prefix,
            resource.full_name().yellow(),
            resource.file_path.display().to_string().dimmed()
        );
    }

    pub fn print_command(command: &str) {
        println!("\n{} {}", "Executing:".bright_blue(), command.white());
    }

    pub fn print_success(message: &str) {
        println!("{} {}", "Success:".green().bold(), message);
    }
}

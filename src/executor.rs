use ctrlc;
use log::{debug, error};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::cli::Operation;
use crate::display::Display;
use crate::error::{Result, TfocusError};
use crate::selector::{SelectItem, Selector};
use crate::types::Resource;

static mut CHILD_PID: Option<u32> = None;

pub fn execute_with_resources(resources: &[Resource]) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        unsafe {
            if let Some(pid) = CHILD_PID {
                Display::print_header("\nReceived Ctrl+C, terminating...");
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;
                    let _ = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
                }
                #[cfg(windows)]
                {
                    use windows::Win32::Foundation::HANDLE;
                    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess};
                }
            }
        }
    })
    .map_err(|e| TfocusError::CommandExecutionError(e.to_string()))?;

    let target_options: Vec<String> = resources
        .iter()
        .map(|r| {
            if r.is_module {
                format!("-target=module.{}", r.name)
            } else {
                format!("-target=resource.{}.{}", r.resource_type, r.name)
            }
        })
        .collect();

    if target_options.is_empty() {
        return Err(TfocusError::ParseError("No targets specified".to_string()));
    }

    Display::print_header("Select operation:");

    let items = vec![
        SelectItem {
            display: "plan  - Show changes to be made".to_string(),
            search_text: "plan terraform show changes".to_string(),
            data: "1".to_string(),
        },
        SelectItem {
            display: "apply - Execute the planned changes".to_string(),
            search_text: "apply terraform execute changes".to_string(),
            data: "2".to_string(),
        },
    ];

    let mut selector = Selector::new(items);
    let operation = match selector.run()? {
        Some(input) => match input.as_str() {
            "1" => Operation::Plan,
            "2" => Operation::Apply,
            _ => return Err(TfocusError::InvalidOperation(input)),
        },
        None => {
            println!("\nOperation cancelled");
            return Ok(());
        }
    };

    execute_terraform_command(&operation, &target_options, running)
}

fn execute_terraform_command(
    operation: &Operation,
    target_options: &[String],
    running: Arc<AtomicBool>,
) -> Result<()> {
    // Build terraform command with basic arguments
    let mut command = Command::new("terraform");
    command.arg(operation.to_string());

    // Add target options
    for target in target_options {
        command.arg(target);
    }

    // Add -auto-approve for apply operations
    if matches!(operation, Operation::Apply) {
        command.arg("-auto-approve");
    }

    // Get the full command string for display
    let command_str = format!(
        "terraform {} {}{}",
        operation.to_string(),
        target_options.join(" "),
        if matches!(operation, Operation::Apply) {
            " -auto-approve"
        } else {
            ""
        }
    );

    Display::print_command(&command_str);
    debug!("Executing command: {}", command_str);

    // Execute terraform command
    let mut child = command
        .spawn()
        .map_err(|e| TfocusError::CommandExecutionError(e.to_string()))?;

    // Store child process ID for signal handling
    unsafe {
        CHILD_PID = Some(child.id());
    }

    // Wait for command completion
    match child.wait() {
        Ok(status) if status.success() => {
            if running.load(Ordering::SeqCst) {
                debug!("Terraform command executed successfully");
                Display::print_success("Operation completed successfully");
                Ok(())
            } else {
                Display::print_header("\nOperation cancelled by user");
                Ok(())
            }
        }
        Ok(status) => {
            let error_msg = format!("Terraform command failed with status: {}", status);
            error!("{}", error_msg);
            Err(TfocusError::TerraformError(error_msg))
        }
        Err(e) => {
            let error_msg = format!("Failed to execute terraform command: {}", e);
            error!("{}", error_msg);
            Err(TfocusError::CommandExecutionError(error_msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_target_option_generation() {
        let resources = vec![
            Resource {
                resource_type: "aws_instance".to_string(),
                name: "web".to_string(),
                is_module: false,
                file_path: PathBuf::from("main.tf"),
            },
            Resource {
                resource_type: String::new(),
                name: "vpc".to_string(),
                is_module: true,
                file_path: PathBuf::from("main.tf"),
            },
        ];

        let target_options: Vec<String> = resources
            .iter()
            .map(|r| {
                if r.is_module {
                    format!("-target=module.{}", r.name)
                } else {
                    format!("-target=resource.{}.{}", r.resource_type, r.name)
                }
            })
            .collect();

        assert_eq!(target_options[0], "-target=resource.aws_instance.web");
        assert_eq!(target_options[1], "-target=module.vpc");
    }
}

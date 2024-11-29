use ctrlc;
use log::{debug, error};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::cli::Operation;
use crate::display::Display;
use crate::error::{Result, TfocusError};
use crate::selector::{SelectItem, Selector};
use crate::types::Resource;

/// Stores the child process ID for signal handling
static mut CHILD_PID: Option<u32> = None;

/// Main entry point for executing Terraform commands on selected resources
pub fn execute_with_resources(resources: &[Resource]) -> Result<()> {
    let running = setup_signal_handler()?;
    let target_options = create_target_options(resources)?;
    let operation = select_operation()?;
    let working_dir = get_working_directory(resources)?;

    let result =
        execute_terraform_command(&operation, &target_options, working_dir, running.clone())?;

    // If plan was successful, suggest terraform apply with the same targets
    if result && matches!(operation, Operation::Plan) {
        Display::print_header("\nTo apply these changes, run:");
        let terraform_command = format!("terraform apply {}", target_options.join(" "));
        println!("  {}", terraform_command);
    }

    Ok(())
}

/// Sets up the Ctrl+C signal handler
fn setup_signal_handler() -> Result<Arc<AtomicBool>> {
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

    Ok(running)
}

/// Creates target options for the Terraform command
fn create_target_options(resources: &[Resource]) -> Result<Vec<String>> {
    let target_options: Vec<String> = resources
        .iter()
        .map(|r| format!("-target={}", r.target_string()))
        .collect();

    if target_options.is_empty() {
        return Err(TfocusError::ParseError("No targets specified".to_string()));
    }

    Ok(target_options)
}

/// Prompts the user to select an operation (plan or apply)
fn select_operation() -> Result<Operation> {
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
    match selector.run()? {
        Some(input) => match input.as_str() {
            "1" => Ok(Operation::Plan),
            "2" => Ok(Operation::Apply),
            _ => Err(TfocusError::InvalidOperation(input)),
        },
        None => {
            println!("\nOperation cancelled");
            std::process::exit(0);
        }
    }
}

/// Gets the working directory from the first resource
fn get_working_directory(resources: &[Resource]) -> Result<&Path> {
    resources
        .first()
        .map(|r| r.file_path.parent().unwrap_or(Path::new(".")))
        .ok_or_else(|| TfocusError::ParseError("No resources specified".to_string()))
}

/// Executes the Terraform command with the specified options
fn execute_terraform_command(
    operation: &Operation,
    target_options: &[String],
    working_dir: &Path,
    running: Arc<AtomicBool>,
) -> Result<bool> {
    let mut command = Command::new("terraform");
    command.arg(operation.to_string()).current_dir(working_dir);

    for target in target_options {
        command.arg(target);
    }

    if matches!(operation, Operation::Apply) {
        command.arg("-auto-approve");
    }

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
    debug!(
        "Executing terraform command in directory: {:?}",
        working_dir
    );
    debug!("Full command: {:?}", command);

    let mut child = command
        .spawn()
        .map_err(|e| TfocusError::CommandExecutionError(e.to_string()))?;

    unsafe {
        CHILD_PID = Some(child.id());
    }

    match child.wait() {
        Ok(status) if status.success() => {
            if running.load(Ordering::SeqCst) {
                debug!("Terraform command executed successfully");
                Display::print_success("Operation completed successfully");
                Ok(true)
            } else {
                Display::print_header("\nOperation cancelled by user");
                Ok(false)
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
    fn test_create_target_options() {
        let resources = vec![
            Resource {
                resource_type: "aws_instance".to_string(),
                name: "web".to_string(),
                is_module: false,
                file_path: PathBuf::from("main.tf"),
                has_count: false,
                has_for_each: false,
                index: None,
            },
            Resource {
                resource_type: "aws_instance".to_string(),
                name: "app".to_string(),
                is_module: false,
                file_path: PathBuf::from("main.tf"),
                has_count: true,
                has_for_each: false,
                index: Some("0".to_string()),
            },
        ];

        let options = create_target_options(&resources).unwrap();
        assert_eq!(options[0], "-target=aws_instance.web");
        assert_eq!(options[1], "-target=aws_instance.app[0]");
    }
}

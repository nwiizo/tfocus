use crate::error::{Result, TfocusError};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub struct InputHandler {
    editor: DefaultEditor,
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        Ok(Self {
            editor: DefaultEditor::new()
                .map_err(|e| TfocusError::CommandExecutionError(e.to_string()))?,
        })
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<String> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to history only if the line is not empty
                if !line.trim().is_empty() {
                    let _ = self.editor.add_history_entry(line.as_str());
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                println!("\nOperation cancelled by user");
                std::process::exit(0);
            }
            Err(ReadlineError::Eof) => {
                println!("\nOperation cancelled by user");
                std::process::exit(0);
            }
            Err(err) => Err(TfocusError::CommandExecutionError(err.to_string())),
        }
    }

    pub fn read_number(&mut self, prompt: &str, max: usize) -> Result<usize> {
        loop {
            let input = self.read_line(prompt)?;
            match input.trim().parse::<usize>() {
                Ok(num) if num > 0 && num <= max => return Ok(num),
                _ => println!("Please enter a number between 1 and {}", max),
            }
        }
    }

    pub fn read_operation(&mut self) -> Result<String> {
        loop {
            let input = self.read_line("\nEnter option (1 or 2): ")?;
            match input.trim() {
                "1" | "2" => return Ok(input),
                _ => println!("Please enter 1 for plan or 2 for apply"),
            }
        }
    }
}

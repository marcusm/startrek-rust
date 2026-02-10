//! Input/output abstractions
//!
//! Provides traits for input and output operations, enabling testing
//! by allowing mock implementations.

use std::io::{self, Write};

/// Trait for reading user input
pub trait InputReader {
    /// Read a line of input from the user with a prompt
    fn read_line(&mut self, prompt: &str) -> Result<String, io::Error>;
}

/// Trait for writing output to the user
pub trait OutputWriter {
    /// Write a message without a newline
    fn write(&mut self, message: &str);
    /// Write a message with a newline
    fn writeln(&mut self, message: &str);
}

/// Terminal I/O implementation using stdin/stdout
pub struct TerminalIO;

impl InputReader for TerminalIO {
    fn read_line(&mut self, prompt: &str) -> Result<String, io::Error> {
        print!("{} ", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input)
    }
}

impl OutputWriter for TerminalIO {
    fn write(&mut self, message: &str) {
        print!("{}", message);
    }

    fn writeln(&mut self, message: &str) {
        println!("{}", message);
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::collections::VecDeque;

    /// Mock input reader for testing
    pub struct MockInput {
        responses: VecDeque<String>,
    }

    impl MockInput {
        pub fn new(responses: Vec<&str>) -> Self {
            Self {
                responses: responses.into_iter().map(|s| s.to_string()).collect(),
            }
        }
    }

    impl InputReader for MockInput {
        fn read_line(&mut self, _prompt: &str) -> Result<String, io::Error> {
            self.responses
                .pop_front()
                .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "No more mock responses"))
        }
    }

    /// Mock output writer for testing
    pub struct MockOutput {
        pub messages: Vec<String>,
    }

    impl MockOutput {
        pub fn new() -> Self {
            Self {
                messages: Vec::new(),
            }
        }
    }

    impl OutputWriter for MockOutput {
        fn write(&mut self, message: &str) {
            self.messages.push(message.to_string());
        }

        fn writeln(&mut self, message: &str) {
            self.messages.push(format!("{}\n", message));
        }
    }
}

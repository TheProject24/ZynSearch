use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum LogCommand {
    AddDocument { doc_id: u32, content: String },
}

pub struct WriteAheadLog {
    file: File,
}

impl WriteAheadLog {
    pub fn new(file_path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(file_path)?;

        Ok(WriteAheadLog { file })
    }

    pub fn write_instruction(&mut self, command: &LogCommand) -> io::Result<()> {
        let mut json_text = serde_json::to_string(command)?;
        json_text.push('\n');
        self.file.write_all(json_text.as_bytes())?;
        self.file.sync_all()?;

        Ok(())
    }

    pub fn recover_instructions(file_path: &Path) -> io::Result<Vec<LogCommand>> {
        let file = File::open(file_path)?;

        let reader = BufReader::new(file);

        let mut recovered_commands = Vec::new();

        for line_result in reader.lines() {
            let line_text = line_result?;

            if let Ok(command) = serde_json::from_str(&line_text) {
                recovered_commands.push(command);
            }
        }

        Ok
    }
}
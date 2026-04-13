use std::{
    cell::RefCell,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    rc::Rc,
};

use crate::{
    error::SchemeError,
    reader::{Datum, Reader},
};

pub type PortRef = Rc<RefCell<Port>>;

#[derive(Clone, Debug)]
pub struct Port {
    kind: PortKind,
    open: bool,
}

#[derive(Clone, Debug)]
enum PortKind {
    Input { source: String, position: usize },
    InputByteVector { bytes: Vec<u8>, position: usize },
    OutputString { buffer: String },
    OutputByteVector { buffer: Vec<u8> },
    OutputFile { path: PathBuf },
    StdInput { source: String, position: usize },
    StdOutput,
}

impl Port {
    pub fn stdin() -> PortRef {
        Rc::new(RefCell::new(Self {
            kind: PortKind::StdInput {
                source: String::new(),
                position: 0,
            },
            open: true,
        }))
    }

    pub fn stdout() -> PortRef {
        Rc::new(RefCell::new(Self {
            kind: PortKind::StdOutput,
            open: true,
        }))
    }

    pub fn open_input_string(source: &str) -> Result<PortRef, SchemeError> {
        Ok(Rc::new(RefCell::new(Self {
            kind: PortKind::Input {
                source: source.to_string(),
                position: 0,
            },
            open: true,
        })))
    }

    pub fn open_output_string() -> PortRef {
        Rc::new(RefCell::new(Self {
            kind: PortKind::OutputString {
                buffer: String::new(),
            },
            open: true,
        }))
    }

    pub fn open_input_bytevector(bytes: &[u8]) -> PortRef {
        Rc::new(RefCell::new(Self {
            kind: PortKind::InputByteVector {
                bytes: bytes.to_vec(),
                position: 0,
            },
            open: true,
        }))
    }

    pub fn open_output_bytevector() -> PortRef {
        Rc::new(RefCell::new(Self {
            kind: PortKind::OutputByteVector { buffer: Vec::new() },
            open: true,
        }))
    }

    pub fn open_input_file(path: &str) -> Result<PortRef, SchemeError> {
        let source = fs::read_to_string(path)
            .map_err(|err| SchemeError::io(format!("failed to read '{path}': {err}")))?;
        Ok(Rc::new(RefCell::new(Self {
            kind: PortKind::Input {
                source,
                position: 0,
            },
            open: true,
        })))
    }

    pub fn open_output_file(path: &str) -> Result<PortRef, SchemeError> {
        fs::write(path, "")
            .map_err(|err| SchemeError::io(format!("failed to create '{path}': {err}")))?;
        Ok(Rc::new(RefCell::new(Self {
            kind: PortKind::OutputFile {
                path: PathBuf::from(path),
            },
            open: true,
        })))
    }

    pub fn read_datum(&mut self) -> Result<Option<Datum>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::Input { source, position } | PortKind::StdInput { source, position } => {
                let remaining = remaining_source(source, *position);
                match Reader::new(&remaining).read_one()? {
                    Some((datum, consumed)) => {
                        *position += consumed;
                        Ok(Some(datum))
                    }
                    None => Ok(None),
                }
            }
            _ => Err(SchemeError::type_error("expected an input port for 'read'")),
        }
    }

    pub fn read_char(&mut self) -> Result<Option<char>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::Input { source, position } | PortKind::StdInput { source, position } => {
                let ch = source.chars().nth(*position);
                if ch.is_some() {
                    *position += 1;
                }
                Ok(ch)
            }
            _ => Err(SchemeError::type_error(
                "expected an input port for character reading",
            )),
        }
    }

    pub fn peek_char(&mut self) -> Result<Option<char>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::Input { source, position } | PortKind::StdInput { source, position } => {
                Ok(source.chars().nth(*position))
            }
            _ => Err(SchemeError::type_error(
                "expected an input port for character reading",
            )),
        }
    }

    pub fn char_ready(&self) -> Result<bool, SchemeError> {
        self.ensure_open()?;
        match &self.kind {
            PortKind::Input { source, position } | PortKind::StdInput { source, position } => {
                Ok(source.chars().nth(*position).is_some())
            }
            _ => Err(SchemeError::type_error(
                "expected a textual input port for 'char-ready?'",
            )),
        }
    }

    pub fn read_string(&mut self, count: usize) -> Result<Option<String>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::Input { source, position } | PortKind::StdInput { source, position } => {
                if count == 0 {
                    return Ok(Some(String::new()));
                }
                let chars = source
                    .chars()
                    .skip(*position)
                    .take(count)
                    .collect::<Vec<_>>();
                if chars.is_empty() {
                    return Ok(None);
                }
                *position += chars.len();
                Ok(Some(chars.into_iter().collect()))
            }
            _ => Err(SchemeError::type_error(
                "expected a textual input port for 'read-string'",
            )),
        }
    }

    pub fn read_bytes(&mut self, count: usize) -> Result<Option<Vec<u8>>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::InputByteVector { bytes, position } => {
                if count == 0 {
                    return Ok(Some(Vec::new()));
                }
                if *position >= bytes.len() {
                    return Ok(None);
                }
                let end = (*position + count).min(bytes.len());
                let chunk = bytes[*position..end].to_vec();
                *position = end;
                Ok(Some(chunk))
            }
            _ => Err(SchemeError::type_error(
                "expected a binary input port for bytevector reading",
            )),
        }
    }

    pub fn read_u8(&mut self) -> Result<Option<u8>, SchemeError> {
        Ok(self
            .read_bytes(1)?
            .and_then(|bytes| bytes.into_iter().next()))
    }

    pub fn peek_u8(&mut self) -> Result<Option<u8>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::InputByteVector { bytes, position } => Ok(bytes.get(*position).copied()),
            _ => Err(SchemeError::type_error(
                "expected a binary input port for 'peek-u8'",
            )),
        }
    }

    pub fn u8_ready(&self) -> Result<bool, SchemeError> {
        self.ensure_open()?;
        match &self.kind {
            PortKind::InputByteVector { bytes, position } => Ok(bytes.get(*position).is_some()),
            _ => Err(SchemeError::type_error(
                "expected a binary input port for 'u8-ready?'",
            )),
        }
    }

    pub fn read_bytes_into(&mut self, target: &mut [u8]) -> Result<Option<usize>, SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::InputByteVector { bytes, position } => {
                if target.is_empty() {
                    return Ok(Some(0));
                }
                if *position >= bytes.len() {
                    return Ok(None);
                }
                let end = (*position + target.len()).min(bytes.len());
                let count = end - *position;
                target[..count].copy_from_slice(&bytes[*position..end]);
                *position = end;
                Ok(Some(count))
            }
            _ => Err(SchemeError::type_error(
                "expected a binary input port for bytevector reading",
            )),
        }
    }

    pub fn write_str(&mut self, text: &str) -> Result<(), SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::OutputString { buffer } => {
                buffer.push_str(text);
                Ok(())
            }
            PortKind::OutputFile { path } => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path.as_path())
                    .map_err(|err| {
                        SchemeError::io(format!(
                            "failed to open '{}' for append: {err}",
                            path.display()
                        ))
                    })?;
                file.write_all(text.as_bytes()).map_err(|err| {
                    SchemeError::io(format!("failed to write '{}': {err}", path.display()))
                })
            }
            PortKind::StdOutput => {
                print!("{text}");
                Ok(())
            }
            PortKind::OutputByteVector { .. } => Err(SchemeError::type_error(
                "expected a textual output port for string writing",
            )),
            _ => Err(SchemeError::type_error(
                "expected an output port for writing",
            )),
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SchemeError> {
        self.ensure_open()?;
        match &mut self.kind {
            PortKind::OutputByteVector { buffer } => {
                buffer.extend_from_slice(bytes);
                Ok(())
            }
            _ => Err(SchemeError::type_error(
                "expected a binary output port for bytevector writing",
            )),
        }
    }

    pub fn flush(&mut self) -> Result<(), SchemeError> {
        self.ensure_open()?;
        match &self.kind {
            PortKind::OutputString { .. }
            | PortKind::OutputByteVector { .. }
            | PortKind::OutputFile { .. }
            | PortKind::StdOutput => Ok(()),
            _ => Err(SchemeError::type_error(
                "expected an output port for 'flush-output-port'",
            )),
        }
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn is_input(&self) -> bool {
        matches!(
            self.kind,
            PortKind::Input { .. } | PortKind::InputByteVector { .. } | PortKind::StdInput { .. }
        )
    }

    pub fn is_output(&self) -> bool {
        matches!(
            self.kind,
            PortKind::OutputString { .. }
                | PortKind::OutputByteVector { .. }
                | PortKind::OutputFile { .. }
                | PortKind::StdOutput
        )
    }

    pub fn is_textual(&self) -> bool {
        matches!(
            self.kind,
            PortKind::Input { .. }
                | PortKind::OutputString { .. }
                | PortKind::OutputFile { .. }
                | PortKind::StdInput { .. }
                | PortKind::StdOutput
        )
    }

    pub fn is_binary(&self) -> bool {
        matches!(
            self.kind,
            PortKind::InputByteVector { .. } | PortKind::OutputByteVector { .. }
        )
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn output_string(&self) -> Result<String, SchemeError> {
        match &self.kind {
            PortKind::OutputString { buffer } => Ok(buffer.clone()),
            _ => Err(SchemeError::type_error(
                "'get-output-string' expected a string output port",
            )),
        }
    }

    pub fn output_bytevector(&self) -> Result<Vec<u8>, SchemeError> {
        match &self.kind {
            PortKind::OutputByteVector { buffer } => Ok(buffer.clone()),
            _ => Err(SchemeError::type_error(
                "'get-output-bytevector' expected a bytevector output port",
            )),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self.kind {
            PortKind::Input { .. } | PortKind::StdInput { .. } => "input-port",
            PortKind::InputByteVector { .. } => "binary-input-port",
            PortKind::OutputString { .. } | PortKind::OutputFile { .. } | PortKind::StdOutput => {
                "output-port"
            }
            PortKind::OutputByteVector { .. } => "binary-output-port",
        }
    }

    fn ensure_open(&self) -> Result<(), SchemeError> {
        if self.open {
            Ok(())
        } else {
            Err(SchemeError::io("port is closed"))
        }
    }
}

fn remaining_source(source: &str, position: usize) -> String {
    source.chars().skip(position).collect()
}

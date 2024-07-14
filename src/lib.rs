use std::cmp::max;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::string::ToString;

use serde::de::{Deserializer, Error as SerdeError, Visitor};
use serde::Deserialize;

/// Represents a `compile_commands.json` file
pub type CompilationDatabase = Vec<CompileCommand>;

/// `All` if `CompilationDatabase` is generated from a `compile_flags.txt` file,
/// otherwise `File()` containing the `file` field from a `compile_commands.json`
/// entry
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum SourceFile {
    All,
    File(PathBuf),
}

impl<'de> Deserialize<'de> for SourceFile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SourceFileVisitor;

        impl<'de> Visitor<'de> for SourceFileVisitor {
            type Value = SourceFile;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a file path")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                Ok(SourceFile::File(PathBuf::from(value)))
            }
        }

        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(s) => Ok(SourceFile::File(PathBuf::from(s))),
            _ => Err(SerdeError::custom("expected a string")),
        }
    }
}

/// Represents a single entry within a `compile_commands.json` file
/// Either `arguments` or `command` is required. `arguments` is preferred, as shell (un)escaping
/// is a possible source of errors.
///
/// See: <https://clang.llvm.org/docs/JSONCompilationDatabase.html#format>
#[derive(Debug, Clone, Deserialize)]
pub struct CompileCommand {
    /// The working directory of the compilation. All paths specified in the `command`
    /// or `file` fields must be either absolute or relative to this directory.
    pub directory: PathBuf,
    /// The main translation unit source processed by this compilation step. This
    /// is used by tools as the key into the compilation database. There can be
    /// multiple command objects for the same file, for example if the same source
    /// file is compiled with different configurations.
    pub file: SourceFile,
    /// The compile command argv as list of strings. This should run the compilation
    /// step for the translation unit file. arguments[0] should be the executable
    /// name, such as clang++. Arguments should not be escaped, but ready to pass
    /// to execvp().
    pub arguments: Option<Vec<String>>,
    /// The compile command as a single shell-escaped string. Arguments may be
    /// shell quoted and escaped following platform conventions, with ‘"’ and ‘\’
    /// being the only special characters. Shell expansion is not supported.
    pub command: Option<String>,
    /// The name of the output created by this compilation step. This field is optional.
    /// It can be used to distinguish different processing modes of the same input
    /// file.
    pub output: Option<PathBuf>,
}

impl Display for CompileCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{ \"directory\": \"{}\",", self.directory.display())?;

        if let Some(arguments) = &self.arguments {
            write!(f, "\"arguments\": [")?;
            if arguments.is_empty() {
                writeln!(f, "],")?;
            } else {
                for arg in arguments.iter().take(arguments.len() - 1) {
                    writeln!(f, "\"{arg}\", ")?;
                }
                writeln!(f, "\"{}\"],", arguments[arguments.len() - 1])?;
            }
        }

        if let Some(command) = &self.command {
            write!(f, "\"command\": \"{command}\"")?;
        }

        if let Some(output) = &self.output {
            writeln!(f, "\"output\": \"{}\"", output.display())?;
        }

        match &self.file {
            SourceFile::All => write!(f, "\"file\": all }}")?,
            SourceFile::File(file) => write!(f, "\"file\": \"{}\" }}", file.display())?,
        }

        Ok(())
    }
}

impl CompileCommand {
    /// Transforms the command field, if present, into a `Vec<String>` of equivalent
    /// arguments
    ///
    /// Replaces escaped '"' and '\' characters with their respective literals
    pub fn args_from_cmd(&self) -> Option<Vec<String>> {
        let escaped = if let Some(ref cmd) = self.command {
            // "Arguments may be shell quoted and escaped following platform conventions,
            // with ‘"’ and ‘\’ being the only special characters."
            cmd.trim().replace("\\\\", "\\").replace("\\\"", "\"")
        } else {
            return None;
        };

        let mut args = Vec::new();
        let mut start: usize = 0;
        let mut end: usize = 0;
        let mut in_quotes = false;

        for c in escaped.chars() {
            if c == '"' {
                in_quotes = !in_quotes;
                end += 1;
            } else if c.is_whitespace() && !in_quotes && start != end {
                args.push(escaped[start..end].to_string());
                end += 1;
                start = end;
            } else {
                end += 1;
            }
        }

        end = max(end, escaped.len() - 1);
        if start != end {
            args.push(escaped[start..end].to_string());
        }

        Some(args)
    }
}

// NOTE: Need to write HELLA tests for this

/// For simple projects, Clang tools also recognize a `compile_flags.txt` file.
/// This should contain one argument per line. The same flags will be used to
/// compile any file.
///
/// See: <https://clang.llvm.org/docs/JSONCompilationDatabase.html#alternatives>
///
/// This helper allows you to translate the contents of a `compile_flags.txt` file
/// to a `CompilationDatabase` object
#[must_use]
pub fn from_compile_flags_txt(directory: &Path, contents: &str) -> CompilationDatabase {
    let args = contents.lines().map(ToString::to_string).collect();
    vec![CompileCommand {
        directory: directory.to_path_buf(),
        file: SourceFile::All,
        arguments: Some(args),
        command: None,
        output: None,
    }]
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: Write tests for:
    //      - from_compile_flags_txt
    //      - args_from_cmd
}

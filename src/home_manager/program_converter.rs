//! Program converter that manages dotfile to Home Manager conversions

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::converter::{DotfileConverter, GitConverter, TmuxConverter, VimConverter, ZshConverter};
use super::ProgramConfig;
use crate::NixDomainError;

/// Manages converters for different programs and orchestrates dotfile conversions
pub struct ProgramConverter {
    converters: HashMap<String, Arc<dyn DotfileConverter>>,
}

impl ProgramConverter {
    /// Create a new program converter with built-in converters
    pub fn new() -> Self {
        let mut converters: HashMap<String, Arc<dyn DotfileConverter>> = HashMap::new();

        // Register built-in converters
        let git_converter = Arc::new(GitConverter::new());
        converters.insert("git".to_string(), git_converter.clone());
        converters.insert("gitconfig".to_string(), git_converter);

        let vim_converter = Arc::new(VimConverter::new());
        converters.insert("vim".to_string(), vim_converter.clone());
        converters.insert("vimrc".to_string(), vim_converter);

        let zsh_converter = Arc::new(ZshConverter::new());
        converters.insert("zsh".to_string(), zsh_converter.clone());
        converters.insert("zshrc".to_string(), zsh_converter);

        let tmux_converter = Arc::new(TmuxConverter::new());
        converters.insert("tmux".to_string(), tmux_converter.clone());
        converters.insert("tmux.conf".to_string(), tmux_converter);

        Self { converters }
    }

    /// Convert a dotfile to a Home Manager program configuration
    pub fn convert(&self, program: &str, dotfile: &Path) -> Result<ProgramConfig, NixDomainError> {
        self.converters
            .get(program)
            .ok_or_else(|| {
                NixDomainError::ParseError(format!("No converter for program: {program}"))
            })?
            .convert(dotfile)
    }

    /// Automatically detect and convert a dotfile
    pub fn auto_convert(&self, dotfile: &Path) -> Result<(String, ProgramConfig), NixDomainError> {
        // Try each converter to see if it can handle the file
        for converter in self.converters.values() {
            if converter.can_handle(dotfile) {
                let config = converter.convert(dotfile)?;
                return Ok((converter.program_name().to_string(), config));
            }
        }

        Err(NixDomainError::ParseError(format!(
            "No converter found for file: {}",
            dotfile.display()
        )))
    }

    /// Register a custom converter
    pub fn register_converter(&mut self, name: String, converter: Arc<dyn DotfileConverter>) {
        self.converters.insert(name, converter);
    }

    /// Get a list of supported programs
    pub fn supported_programs(&self) -> Vec<String> {
        self.converters.keys().cloned().collect()
    }

    /// Check if a program is supported
    pub fn is_supported(&self, program: &str) -> bool {
        self.converters.contains_key(program)
    }
}

impl Default for ProgramConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_program_converter_initialization() {
        let converter = ProgramConverter::new();
        assert!(converter.is_supported("git"));
        assert!(converter.is_supported("vim"));
        assert!(converter.is_supported("zsh"));
        assert!(converter.is_supported("tmux"));
    }

    #[test]
    fn test_git_conversion() {
        let converter = ProgramConverter::new();
        let temp_dir = TempDir::new().unwrap();
        let git_config = temp_dir.path().join(".gitconfig");

        fs::write(
            &git_config,
            r#"
[user]
    name = Test User
    email = test@example.com
[core]
    editor = vim
"#,
        )
        .unwrap();

        let result = converter.convert("git", &git_config);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.enabled);
        assert!(config.settings.contains_key("userName"));
        assert!(config.settings.contains_key("userEmail"));
    }

    #[test]
    fn test_auto_convert() {
        let converter = ProgramConverter::new();
        let temp_dir = TempDir::new().unwrap();
        let vim_config = temp_dir.path().join(".vimrc");

        fs::write(&vim_config, "set number\nsyntax on").unwrap();

        let result = converter.auto_convert(&vim_config);
        assert!(result.is_ok());

        let (program, config) = result.unwrap();
        assert_eq!(program, "vim");
        assert!(config.enabled);
        assert!(config.extra_config.is_some());
    }
}

//! Dotfile to Home Manager converter trait and implementations

use std::path::Path;
use std::fs;

use super::ProgramConfig;
use crate::value_objects::NixValue;
use crate::NixDomainError;

/// Trait for converting dotfiles to Home Manager configuration
pub trait DotfileConverter: Send + Sync {
    /// Convert a dotfile to a Home Manager program configuration
    fn convert(&self, dotfile_path: &Path) -> Result<ProgramConfig, NixDomainError>;
    
    /// Get the program name this converter handles
    fn program_name(&self) -> &str;
    
    /// Check if this converter can handle the given dotfile
    fn can_handle(&self, dotfile_path: &Path) -> bool;
}

/// Git configuration converter
pub struct GitConverter;

impl Default for GitConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl GitConverter {
    /// Create a new Git converter
    pub fn new() -> Self {
        Self
    }
}

impl DotfileConverter for GitConverter {
    fn convert(&self, dotfile_path: &Path) -> Result<ProgramConfig, NixDomainError> {
        let content = fs::read_to_string(dotfile_path)
            .map_err(|e| NixDomainError::ParseError(format!("Failed to read git config: {e}")))?;
        
        let mut config = ProgramConfig::new(true);
        let mut user_settings = std::collections::HashMap::new();
        let mut core_settings = std::collections::HashMap::new();
        
        // Simple INI-style parser for git config
        let mut current_section = "";
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with('[') && line.ends_with(']') {
                current_section = &line[1..line.len()-1];
            } else if line.contains('=') {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let value = parts[1].trim();
                    
                    match current_section {
                        "user" => {
                            user_settings.insert(key.to_string(), NixValue::String(value.to_string()));
                        }
                        "core" => {
                            core_settings.insert(key.to_string(), NixValue::String(value.to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Convert to Home Manager format
        if !user_settings.is_empty() {
            config.settings.insert("userName".to_string(), 
                user_settings.get("name").cloned()
                    .unwrap_or_else(|| NixValue::String(String::new())));
            config.settings.insert("userEmail".to_string(), 
                user_settings.get("email").cloned()
                    .unwrap_or_else(|| NixValue::String(String::new())));
        }
        
        if !core_settings.is_empty() {
            let mut extra_config = std::collections::HashMap::new();
            extra_config.insert("core".to_string(), NixValue::AttrSet(core_settings));
            config.settings.insert("extraConfig".to_string(), NixValue::AttrSet(extra_config));
        }
        
        Ok(config)
    }
    
    fn program_name(&self) -> &'static str {
        "git"
    }
    
    fn can_handle(&self, dotfile_path: &Path) -> bool {
        if let Some(name) = dotfile_path.file_name().and_then(|n| n.to_str()) {
            name == ".gitconfig" || name == "gitconfig"
        } else {
            false
        }
    }
}

/// Vim configuration converter
pub struct VimConverter;

impl Default for VimConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl VimConverter {
    /// Create a new Vim converter
    pub fn new() -> Self {
        Self
    }
}

impl DotfileConverter for VimConverter {
    fn convert(&self, dotfile_path: &Path) -> Result<ProgramConfig, NixDomainError> {
        let content = fs::read_to_string(dotfile_path)
            .map_err(|e| NixDomainError::ParseError(format!("Failed to read vim config: {e}")))?;
        
        let mut config = ProgramConfig::new(true);
        
        // Try to extract some common settings first
        let mut settings = std::collections::HashMap::new();
        
        // Check for common patterns
        if content.contains("set number") {
            settings.insert("number".to_string(), NixValue::Bool(true));
        }
        if content.contains("set relativenumber") {
            settings.insert("relativenumber".to_string(), NixValue::Bool(true));
        }
        if content.contains("syntax on") {
            settings.insert("syntax".to_string(), NixValue::Bool(true));
        }
        
        if !settings.is_empty() {
            config.settings.insert("settings".to_string(), NixValue::AttrSet(settings));
        }
        
        // For Vim, we'll preserve the entire config as extraConfig
        config.extra_config = Some(content);
        
        Ok(config)
    }
    
    fn program_name(&self) -> &'static str {
        "vim"
    }
    
    fn can_handle(&self, dotfile_path: &Path) -> bool {
        if let Some(name) = dotfile_path.file_name().and_then(|n| n.to_str()) {
            name == ".vimrc" || name == "vimrc" || name == ".vim"
        } else {
            false
        }
    }
}

/// Zsh configuration converter
pub struct ZshConverter;

impl Default for ZshConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl ZshConverter {
    /// Create a new Zsh converter
    pub fn new() -> Self {
        Self
    }
}

impl DotfileConverter for ZshConverter {
    fn convert(&self, dotfile_path: &Path) -> Result<ProgramConfig, NixDomainError> {
        let content = fs::read_to_string(dotfile_path)
            .map_err(|e| NixDomainError::ParseError(format!("Failed to read zsh config: {e}")))?;
        
        let mut config = ProgramConfig::new(true);
        
        // Extract common zsh configurations
        let mut init_extra = Vec::new();
        let mut aliases = std::collections::HashMap::new();
        let mut env_vars = std::collections::HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("alias ") {
                // Parse alias
                if let Some(alias_def) = line.strip_prefix("alias ") {
                    if let Some(eq_pos) = alias_def.find('=') {
                        let name = alias_def[..eq_pos].trim();
                        let value = alias_def[eq_pos+1..].trim().trim_matches('"').trim_matches('\'');
                        aliases.insert(name.to_string(), NixValue::String(value.to_string()));
                    }
                }
            } else if line.starts_with("export ") {
                // Parse environment variable
                if let Some(export_def) = line.strip_prefix("export ") {
                    if let Some(eq_pos) = export_def.find('=') {
                        let name = export_def[..eq_pos].trim();
                        let value = export_def[eq_pos+1..].trim().trim_matches('"').trim_matches('\'');
                        env_vars.insert(name.to_string(), NixValue::String(value.to_string()));
                    }
                }
            } else if !line.is_empty() && !line.starts_with('#') {
                // Other configuration lines
                init_extra.push(line.to_string());
            }
        }
        
        // Convert to Home Manager format
        if !aliases.is_empty() {
            config.settings.insert("shellAliases".to_string(), NixValue::AttrSet(aliases));
        }
        
        if !env_vars.is_empty() {
            config.settings.insert("sessionVariables".to_string(), NixValue::AttrSet(env_vars));
        }
        
        if !init_extra.is_empty() {
            config.settings.insert("initExtra".to_string(), NixValue::String(init_extra.join("\n")));
        }
        
        Ok(config)
    }
    
    fn program_name(&self) -> &'static str {
        "zsh"
    }
    
    fn can_handle(&self, dotfile_path: &Path) -> bool {
        if let Some(name) = dotfile_path.file_name().and_then(|n| n.to_str()) {
            name == ".zshrc" || name == ".zshenv" || name == "zshrc"
        } else {
            false
        }
    }
}

/// Tmux configuration converter
pub struct TmuxConverter;

impl Default for TmuxConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl TmuxConverter {
    /// Create a new Tmux converter
    pub fn new() -> Self {
        Self
    }
}

impl DotfileConverter for TmuxConverter {
    fn convert(&self, dotfile_path: &Path) -> Result<ProgramConfig, NixDomainError> {
        let content = fs::read_to_string(dotfile_path)
            .map_err(|e| NixDomainError::ParseError(format!("Failed to read tmux config: {e}")))?;
        
        let mut config = ProgramConfig::new(true);
        
        // For tmux, preserve the entire config
        config.extra_config = Some(content.clone());
        
        // Extract some common settings
        let mut settings = std::collections::HashMap::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("set -g prefix") {
                if let Some(prefix) = line.split_whitespace().last() {
                    settings.insert("prefix".to_string(), NixValue::String(prefix.to_string()));
                }
            } else if line.contains("mouse on") {
                settings.insert("mouse".to_string(), NixValue::Bool(true));
            } else if line.contains("base-index 1") {
                settings.insert("baseIndex".to_string(), NixValue::Int(1));
            }
        }
        
        if !settings.is_empty() {
            config.settings = settings;
        }
        
        Ok(config)
    }
    
    fn program_name(&self) -> &'static str {
        "tmux"
    }
    
    fn can_handle(&self, dotfile_path: &Path) -> bool {
        if let Some(name) = dotfile_path.file_name().and_then(|n| n.to_str()) {
            name == ".tmux.conf" || name == "tmux.conf"
        } else {
            false
        }
    }
} 
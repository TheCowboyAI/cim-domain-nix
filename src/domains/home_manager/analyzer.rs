// Copyright 2025 Cowboy AI, LLC.

//! Dotfile analyzer for detecting programs and configurations

use super::value_objects::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Result of dotfile analysis
#[derive(Debug, Clone)]
pub struct DotfileAnalysisResult {
    /// Detected programs and their configurations
    pub detected_programs: Vec<DetectedProgram>,
    /// Shell configuration
    pub shell_config: Option<DetectedShellConfig>,
    /// Git configuration
    pub git_config: Option<DetectedGitConfig>,
    /// Vim/Neovim configuration
    pub editor_config: Option<DetectedEditorConfig>,
    /// Other dotfiles that couldn't be categorized
    pub uncategorized_files: Vec<PathBuf>,
}

/// Detected program from dotfiles
#[derive(Debug, Clone)]
pub struct DetectedProgram {
    /// Program name
    pub name: String,
    /// Config files found
    pub config_files: Vec<PathBuf>,
    /// Suggested Home Manager configuration
    pub suggested_config: ProgramConfig,
}

/// Detected shell configuration
#[derive(Debug, Clone)]
pub struct DetectedShellConfig {
    /// Shell type
    pub shell_type: ShellType,
    /// RC file path
    pub rc_file: PathBuf,
    /// Profile file path (if exists)
    pub profile_file: Option<PathBuf>,
    /// Detected aliases
    pub aliases: HashMap<String, String>,
    /// Detected environment variables
    pub environment: HashMap<String, String>,
}

/// Detected Git configuration
#[derive(Debug, Clone)]
pub struct DetectedGitConfig {
    /// Global gitconfig path
    pub gitconfig: PathBuf,
    /// Gitignore global path
    pub gitignore_global: Option<PathBuf>,
    /// User name
    pub user_name: Option<String>,
    /// User email
    pub user_email: Option<String>,
    /// Aliases
    pub aliases: HashMap<String, String>,
}

/// Detected editor configuration
#[derive(Debug, Clone)]
pub struct DetectedEditorConfig {
    /// Editor type
    pub editor_type: EditorType,
    /// Main config file
    pub config_file: PathBuf,
    /// Plugin manager detected
    pub plugin_manager: Option<String>,
    /// Additional config files
    pub additional_files: Vec<PathBuf>,
}

/// Editor types
#[derive(Debug, Clone, PartialEq)]
pub enum EditorType {
    /// Vim text editor
    Vim,
    /// Neovim modern Vim fork
    Neovim,
    /// Emacs text editor
    Emacs,
    /// Visual Studio Code
    VSCode,
    /// Other editor with custom name
    Other(String),
}

/// Dotfile analyzer
pub struct DotfileAnalyzer {
    /// Base path for dotfiles
    base_path: PathBuf,
    /// Include patterns
    include_patterns: Vec<String>,
    /// Exclude patterns
    exclude_patterns: Vec<String>,
}

impl DotfileAnalyzer {
    /// Create a new dotfile analyzer
    pub fn new(
        base_path: PathBuf,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    ) -> Self {
        Self {
            base_path,
            include_patterns,
            exclude_patterns,
        }
    }

    /// Analyze dotfiles
    pub fn analyze(&self) -> Result<DotfileAnalysisResult, String> {
        let mut result = DotfileAnalysisResult {
            detected_programs: Vec::new(),
            shell_config: None,
            git_config: None,
            editor_config: None,
            uncategorized_files: Vec::new(),
        };

        // Scan dotfiles
        let files = self.scan_dotfiles()?;

        for file in files {
            let file_name = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            match file_name {
                // Shell configurations
                ".bashrc" | ".bash_profile" | ".bash_aliases" => {
                    result.shell_config = Some(self.analyze_bash_config(&file)?);
                }
                ".zshrc" | ".zprofile" | ".zshenv" => {
                    result.shell_config = Some(self.analyze_zsh_config(&file)?);
                }
                ".config/fish/config.fish" => {
                    result.shell_config = Some(self.analyze_fish_config(&file)?);
                }
                
                // Git configuration
                ".gitconfig" => {
                    result.git_config = Some(self.analyze_git_config(&file)?);
                }
                
                // Editor configurations
                ".vimrc" | ".vim" => {
                    result.editor_config = Some(self.analyze_vim_config(&file)?);
                }
                ".config/nvim" | ".config/nvim/init.vim" | ".config/nvim/init.lua" => {
                    result.editor_config = Some(self.analyze_neovim_config(&file)?);
                }
                ".emacs" | ".emacs.d" => {
                    result.editor_config = Some(self.analyze_emacs_config(&file)?);
                }
                
                // Program-specific configs
                ".tmux.conf" => {
                    result.detected_programs.push(self.analyze_tmux_config(&file)?);
                }
                ".config/alacritty" | ".alacritty.yml" => {
                    result.detected_programs.push(self.analyze_alacritty_config(&file)?);
                }
                ".config/kitty" => {
                    result.detected_programs.push(self.analyze_kitty_config(&file)?);
                }
                ".ssh/config" => {
                    result.detected_programs.push(self.analyze_ssh_config(&file)?);
                }
                
                // Otherwise uncategorized
                _ => {
                    if file_name.starts_with('.') {
                        result.uncategorized_files.push(file);
                    }
                }
            }
        }

        // Convert detected configs to program configs
        if let Some(git_config) = &result.git_config {
            result.detected_programs.push(self.git_config_to_program(git_config));
        }
        if let Some(editor_config) = &result.editor_config {
            result.detected_programs.push(self.editor_config_to_program(editor_config));
        }

        Ok(result)
    }

    /// Scan dotfiles based on patterns
    fn scan_dotfiles(&self) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        self.scan_directory(&self.base_path, &mut files)?;
        Ok(files)
    }

    /// Recursively scan directory
    fn scan_directory(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.base_path)
                .unwrap_or(&path)
                .to_str()
                .unwrap_or("");

            // Check exclude patterns
            if self.matches_patterns(relative_path, &self.exclude_patterns) {
                continue;
            }

            // Check include patterns (if any specified)
            if !self.include_patterns.is_empty() 
                && !self.matches_patterns(relative_path, &self.include_patterns) {
                continue;
            }

            if path.is_dir() {
                // Don't recurse into .git directory
                if path.file_name().map(|n| n == ".git").unwrap_or(false) {
                    continue;
                }
                self.scan_directory(&path, files)?;
            } else {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Check if path matches any of the patterns
    fn matches_patterns(&self, path: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(path))
                .unwrap_or(false)
        })
    }

    /// Analyze Bash configuration
    fn analyze_bash_config(&self, _file: &Path) -> Result<DetectedShellConfig, String> {
        // TODO: Parse bash config file
        Ok(DetectedShellConfig {
            shell_type: ShellType::Bash,
            rc_file: _file.to_path_buf(),
            profile_file: None,
            aliases: HashMap::new(),
            environment: HashMap::new(),
        })
    }

    /// Analyze Zsh configuration
    fn analyze_zsh_config(&self, _file: &Path) -> Result<DetectedShellConfig, String> {
        // TODO: Parse zsh config file
        Ok(DetectedShellConfig {
            shell_type: ShellType::Zsh,
            rc_file: _file.to_path_buf(),
            profile_file: None,
            aliases: HashMap::new(),
            environment: HashMap::new(),
        })
    }

    /// Analyze Fish configuration
    fn analyze_fish_config(&self, _file: &Path) -> Result<DetectedShellConfig, String> {
        // TODO: Parse fish config file
        Ok(DetectedShellConfig {
            shell_type: ShellType::Fish,
            rc_file: _file.to_path_buf(),
            profile_file: None,
            aliases: HashMap::new(),
            environment: HashMap::new(),
        })
    }

    /// Analyze Git configuration
    fn analyze_git_config(&self, _file: &Path) -> Result<DetectedGitConfig, String> {
        // TODO: Parse git config file
        Ok(DetectedGitConfig {
            gitconfig: _file.to_path_buf(),
            gitignore_global: None,
            user_name: None,
            user_email: None,
            aliases: HashMap::new(),
        })
    }

    /// Analyze Vim configuration
    fn analyze_vim_config(&self, _file: &Path) -> Result<DetectedEditorConfig, String> {
        Ok(DetectedEditorConfig {
            editor_type: EditorType::Vim,
            config_file: _file.to_path_buf(),
            plugin_manager: None,
            additional_files: Vec::new(),
        })
    }

    /// Analyze Neovim configuration
    fn analyze_neovim_config(&self, _file: &Path) -> Result<DetectedEditorConfig, String> {
        Ok(DetectedEditorConfig {
            editor_type: EditorType::Neovim,
            config_file: _file.to_path_buf(),
            plugin_manager: None,
            additional_files: Vec::new(),
        })
    }

    /// Analyze Emacs configuration
    fn analyze_emacs_config(&self, _file: &Path) -> Result<DetectedEditorConfig, String> {
        Ok(DetectedEditorConfig {
            editor_type: EditorType::Emacs,
            config_file: _file.to_path_buf(),
            plugin_manager: None,
            additional_files: Vec::new(),
        })
    }

    /// Analyze tmux configuration
    fn analyze_tmux_config(&self, file: &Path) -> Result<DetectedProgram, String> {
        Ok(DetectedProgram {
            name: "tmux".to_string(),
            config_files: vec![file.to_path_buf()],
            suggested_config: ProgramConfig {
                name: "tmux".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            },
        })
    }

    /// Analyze alacritty configuration
    fn analyze_alacritty_config(&self, file: &Path) -> Result<DetectedProgram, String> {
        Ok(DetectedProgram {
            name: "alacritty".to_string(),
            config_files: vec![file.to_path_buf()],
            suggested_config: ProgramConfig {
                name: "alacritty".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            },
        })
    }

    /// Analyze kitty configuration
    fn analyze_kitty_config(&self, file: &Path) -> Result<DetectedProgram, String> {
        Ok(DetectedProgram {
            name: "kitty".to_string(),
            config_files: vec![file.to_path_buf()],
            suggested_config: ProgramConfig {
                name: "kitty".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            },
        })
    }

    /// Analyze SSH configuration
    fn analyze_ssh_config(&self, file: &Path) -> Result<DetectedProgram, String> {
        Ok(DetectedProgram {
            name: "ssh".to_string(),
            config_files: vec![file.to_path_buf()],
            suggested_config: ProgramConfig {
                name: "ssh".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            },
        })
    }

    /// Convert git config to program config
    fn git_config_to_program(&self, config: &DetectedGitConfig) -> DetectedProgram {
        let mut settings = serde_json::json!({});
        
        if let Some(name) = &config.user_name {
            settings["userName"] = serde_json::json!(name);
        }
        if let Some(email) = &config.user_email {
            settings["userEmail"] = serde_json::json!(email);
        }
        if !config.aliases.is_empty() {
            settings["aliases"] = serde_json::json!(config.aliases);
        }

        DetectedProgram {
            name: "git".to_string(),
            config_files: vec![config.gitconfig.clone()],
            suggested_config: ProgramConfig {
                name: "git".to_string(),
                enable: true,
                settings,
                extra_packages: vec![],
            },
        }
    }

    /// Convert editor config to program config
    fn editor_config_to_program(&self, config: &DetectedEditorConfig) -> DetectedProgram {
        let name = match &config.editor_type {
            EditorType::Vim => "vim",
            EditorType::Neovim => "neovim",
            EditorType::Emacs => "emacs",
            EditorType::VSCode => "vscode",
            EditorType::Other(n) => n,
        };

        let mut settings = serde_json::json!({});
        if let Some(pm) = &config.plugin_manager {
            settings["pluginManager"] = serde_json::json!(pm);
        }

        DetectedProgram {
            name: name.to_string(),
            config_files: vec![config.config_file.clone()],
            suggested_config: ProgramConfig {
                name: name.to_string(),
                enable: true,
                settings,
                extra_packages: vec![],
            },
        }
    }
}
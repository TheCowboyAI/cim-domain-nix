// Copyright 2025 Cowboy AI, LLC.

//! Value objects for Home Manager domain

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Unique identifier for a home configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HomeConfigId(
    /// UUID value for the home configuration ID
    pub uuid::Uuid
);

impl HomeConfigId {
    /// Create a new home config ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for HomeConfigId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<uuid::Uuid> for HomeConfigId {
    fn from(id: uuid::Uuid) -> Self {
        Self(id)
    }
}

/// Program configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Program name (e.g., "git", "vim", "zsh")
    pub name: String,
    /// Whether the program is enabled
    pub enable: bool,
    /// Program-specific settings as JSON
    pub settings: serde_json::Value,
    /// Additional packages required by this program
    pub extra_packages: Vec<String>,
}

/// Service configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Whether the service is enabled
    pub enable: bool,
    /// Service-specific settings as JSON
    pub settings: serde_json::Value,
    /// Environment variables for the service
    pub environment: HashMap<String, String>,
}

/// Dotfile entry
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DotfileEntry {
    /// Source path (relative to dotfiles directory)
    pub source: PathBuf,
    /// Target path (relative to home directory)
    pub target: PathBuf,
    /// File mode (e.g., "0644", "0755")
    pub mode: Option<String>,
    /// Whether to create a symlink or copy
    pub symlink: bool,
}

/// Shell configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell type (Bash, Zsh, Fish, etc.)
    pub shell_type: ShellType,
    /// Shell aliases mapping
    pub aliases: HashMap<String, String>,
    /// Shell environment variables
    pub environment: HashMap<String, String>,
    /// Shell initialization script content
    pub init_script: Option<String>,
    /// Interactive shell startup script
    pub interactive_script: Option<String>,
    /// Login shell startup script
    pub login_script: Option<String>,
}

/// Shell types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Z shell
    Zsh,
    /// Fish shell
    Fish,
    /// Nushell
    Nushell,
    /// Other shell type
    Other(String),
}

/// Desktop environment configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Desktop environment type (Gnome, KDE, i3, etc.)
    pub desktop_type: DesktopType,
    /// Window manager configuration if applicable
    pub window_manager: Option<WindowManagerConfig>,
    /// Desktop theme configuration
    pub theme: Option<ThemeConfig>,
    /// Keyboard layout and options
    pub keyboard: Option<KeyboardConfig>,
}

/// Desktop environment types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DesktopType {
    /// GNOME desktop environment
    Gnome,
    /// KDE Plasma desktop environment
    Kde,
    /// XFCE desktop environment
    Xfce,
    /// i3 window manager
    I3,
    /// Sway window manager
    Sway,
    /// Hyprland compositor
    Hyprland,
    /// Other desktop environment
    Other(String),
}

/// Window manager configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowManagerConfig {
    /// Window manager name
    pub name: String,
    /// Window manager configuration content
    pub config: String,
    /// Key bindings mapping
    pub keybindings: HashMap<String, String>,
}

/// Theme configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// GTK theme name
    pub gtk_theme: Option<String>,
    /// Icon theme name
    pub icon_theme: Option<String>,
    /// Cursor theme name
    pub cursor_theme: Option<String>,
    /// Font configuration settings
    pub fonts: FontConfig,
}

/// Font configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontConfig {
    /// Default system font
    pub default: Option<String>,
    /// Monospace font for terminals and editors
    pub monospace: Option<String>,
    /// Default font size in points
    pub size: Option<u32>,
}

/// Keyboard configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardConfig {
    /// Keyboard layout (e.g., "us", "dvorak")
    pub layout: String,
    /// Layout variant if applicable
    pub variant: Option<String>,
    /// Keyboard options (e.g., "ctrl:nocaps")
    pub options: Vec<String>,
}

/// Package set for Home Manager
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageSet {
    /// Core system packages
    pub system: Vec<String>,
    /// Development tools and libraries
    pub development: Vec<String>,
    /// Desktop applications and utilities
    pub desktop: Vec<String>,
    /// User-defined custom packages
    pub custom: Vec<String>,
}

impl Default for PackageSet {
    fn default() -> Self {
        Self {
            system: Vec::new(),
            development: Vec::new(),
            desktop: Vec::new(),
            custom: Vec::new(),
        }
    }
}

/// Migration status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Not started
    NotStarted,
    /// Analyzing existing configuration
    Analyzing,
    /// Generating Home Manager configuration
    Generating,
    /// Validating configuration
    Validating,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
}

/// Configuration source type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Dotfiles in a directory
    Dotfiles(PathBuf),
    /// Existing Home Manager configuration
    HomeManager(PathBuf),
    /// NixOS configuration
    NixOS(PathBuf),
    /// Manual configuration
    Manual,
}

/// User profile information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    /// System username
    pub username: String,
    /// User's full display name
    pub full_name: Option<String>,
    /// User's email address
    pub email: Option<String>,
    /// Path to user's home directory
    pub home_directory: PathBuf,
    /// User's preferred shell
    pub shell: Option<String>,
}
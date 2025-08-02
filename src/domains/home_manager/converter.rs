// Copyright 2025 Cowboy AI, LLC.

//! Configuration converter for generating Home Manager Nix files

use super::handlers::HomeConfigReadModel;
use super::value_objects::*;
use std::collections::HashMap;

/// Home Manager configuration converter
pub struct HomeManagerConverter {
    /// Whether to include comments
    include_comments: bool,
    /// Indentation style
    indent_style: IndentStyle,
}

/// Indentation style
#[derive(Debug, Clone, Copy)]
pub enum IndentStyle {
    /// Spaces with specified count per indent level
    Spaces(usize),
    /// Tab characters for indentation
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(2)
    }
}

impl HomeManagerConverter {
    /// Create a new converter
    pub fn new(include_comments: bool, indent_style: IndentStyle) -> Self {
        Self {
            include_comments,
            indent_style,
        }
    }

    /// Convert a home configuration to Nix expression
    pub fn convert_to_nix(&self, config: &HomeConfigReadModel) -> Result<String, String> {
        let mut nix = String::new();
        
        // Header comment
        if self.include_comments {
            nix.push_str("# Home Manager configuration\n");
            nix.push_str(&format!("# Generated for: {}\n", config.user_profile.username));
            nix.push_str("\n");
        }

        nix.push_str("{ config, pkgs, ... }:\n\n");
        nix.push_str("{\n");

        // Home settings
        self.write_home_settings(&mut nix, &config.user_profile)?;
        
        // Programs
        if !config.programs.is_empty() {
            self.write_programs(&mut nix, &config.programs)?;
        }

        // Services
        if !config.services.is_empty() {
            self.write_services(&mut nix, &config.services)?;
        }

        // Shell configuration
        if let Some(shell) = &config.shell {
            self.write_shell_config(&mut nix, shell)?;
        }

        // Desktop configuration
        if let Some(desktop) = &config.desktop {
            self.write_desktop_config(&mut nix, desktop)?;
        }

        // Packages
        self.write_packages(&mut nix, &config.packages)?;

        // Dotfiles
        if !config.dotfiles.is_empty() {
            self.write_dotfiles(&mut nix, &config.dotfiles)?;
        }

        nix.push_str("}\n");

        Ok(nix)
    }

    /// Write home settings
    fn write_home_settings(&self, nix: &mut String, profile: &UserProfile) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Basic home settings\n", indent));
        }
        
        nix.push_str(&format!("{}home.username = \"{}\";\n", indent, profile.username));
        nix.push_str(&format!("{}home.homeDirectory = \"{}\";\n", 
            indent, 
            profile.home_directory.display()
        ));
        
        if let Some(name) = &profile.full_name {
            nix.push_str(&format!("{}home.sessionVariables.NAME = \"{}\";\n", indent, name));
        }
        
        nix.push_str(&format!("{}home.stateVersion = \"24.05\";\n", indent));
        nix.push_str("\n");

        Ok(())
    }

    /// Write programs configuration
    fn write_programs(&self, nix: &mut String, programs: &HashMap<String, ProgramConfig>) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Program configurations\n", indent));
        }
        
        nix.push_str(&format!("{}programs = {{\n", indent));
        
        for (name, config) in programs {
            self.write_program(nix, name, config, 2)?;
        }
        
        nix.push_str(&format!("{}}};\n\n", indent));

        Ok(())
    }

    /// Write a single program configuration
    fn write_program(&self, nix: &mut String, name: &str, config: &ProgramConfig, indent_level: usize) -> Result<(), String> {
        let indent = self.get_indent(indent_level);
        
        nix.push_str(&format!("{}{} = {{\n", indent, name));
        nix.push_str(&format!("{}enable = {};\n", 
            self.get_indent(indent_level + 1), 
            if config.enable { "true" } else { "false" }
        ));
        
        // Write settings based on program type
        match name {
            "git" => self.write_git_settings(nix, &config.settings, indent_level + 1)?,
            "vim" | "neovim" => self.write_editor_settings(nix, &config.settings, indent_level + 1)?,
            "zsh" | "bash" | "fish" => self.write_shell_program_settings(nix, &config.settings, indent_level + 1)?,
            _ => self.write_generic_settings(nix, &config.settings, indent_level + 1)?,
        }
        
        if !config.extra_packages.is_empty() {
            nix.push_str(&format!("{}extraPackages = with pkgs; [\n", 
                self.get_indent(indent_level + 1)
            ));
            for pkg in &config.extra_packages {
                nix.push_str(&format!("{}{}\n", self.get_indent(indent_level + 2), pkg));
            }
            nix.push_str(&format!("{}];\n", self.get_indent(indent_level + 1)));
        }
        
        nix.push_str(&format!("{}}};\n", indent));

        Ok(())
    }

    /// Write Git-specific settings
    fn write_git_settings(&self, nix: &mut String, settings: &serde_json::Value, indent_level: usize) -> Result<(), String> {
        let indent = self.get_indent(indent_level);
        
        if let Some(user_name) = settings.get("userName").and_then(|v| v.as_str()) {
            nix.push_str(&format!("{}userName = \"{}\";\n", indent, user_name));
        }
        
        if let Some(user_email) = settings.get("userEmail").and_then(|v| v.as_str()) {
            nix.push_str(&format!("{}userEmail = \"{}\";\n", indent, user_email));
        }
        
        if let Some(aliases) = settings.get("aliases").and_then(|v| v.as_object()) {
            nix.push_str(&format!("{}aliases = {{\n", indent));
            for (alias, command) in aliases {
                if let Some(cmd) = command.as_str() {
                    nix.push_str(&format!("{}{} = \"{}\";\n", 
                        self.get_indent(indent_level + 1), 
                        alias, 
                        cmd
                    ));
                }
            }
            nix.push_str(&format!("{}}};\n", indent));
        }

        Ok(())
    }

    /// Write editor settings
    fn write_editor_settings(&self, nix: &mut String, settings: &serde_json::Value, indent_level: usize) -> Result<(), String> {
        let indent = self.get_indent(indent_level);
        
        if let Some(plugin_manager) = settings.get("pluginManager").and_then(|v| v.as_str()) {
            if self.include_comments {
                nix.push_str(&format!("{}# Plugin manager: {}\n", indent, plugin_manager));
            }
        }
        
        self.write_generic_settings(nix, settings, indent_level)
    }

    /// Write shell program settings
    fn write_shell_program_settings(&self, nix: &mut String, settings: &serde_json::Value, indent_level: usize) -> Result<(), String> {
        self.write_generic_settings(nix, settings, indent_level)
    }

    /// Write generic settings as Nix attributes
    fn write_generic_settings(&self, nix: &mut String, settings: &serde_json::Value, indent_level: usize) -> Result<(), String> {
        let indent = self.get_indent(indent_level);
        
        if let Some(obj) = settings.as_object() {
            for (key, value) in obj {
                // Skip special keys we've already handled
                if matches!(key.as_str(), "userName" | "userEmail" | "aliases" | "pluginManager") {
                    continue;
                }
                
                match value {
                    serde_json::Value::String(s) => {
                        nix.push_str(&format!("{}{} = \"{}\";\n", indent, key, s));
                    }
                    serde_json::Value::Bool(b) => {
                        nix.push_str(&format!("{}{} = {};\n", indent, key, b));
                    }
                    serde_json::Value::Number(n) => {
                        nix.push_str(&format!("{}{} = {};\n", indent, key, n));
                    }
                    serde_json::Value::Array(arr) => {
                        nix.push_str(&format!("{}{} = [\n", indent, key));
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                nix.push_str(&format!("{}\"{}\";\n", 
                                    self.get_indent(indent_level + 1), 
                                    s
                                ));
                            }
                        }
                        nix.push_str(&format!("{}];\n", indent));
                    }
                    _ => {
                        // For complex values, convert to string representation
                        nix.push_str(&format!("{}{} = {};\n", indent, key, value));
                    }
                }
            }
        }

        Ok(())
    }

    /// Write services configuration
    fn write_services(&self, nix: &mut String, services: &HashMap<String, ServiceConfig>) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Service configurations\n", indent));
        }
        
        nix.push_str(&format!("{}services = {{\n", indent));
        
        for (name, config) in services {
            self.write_service(nix, name, config, 2)?;
        }
        
        nix.push_str(&format!("{}}};\n\n", indent));

        Ok(())
    }

    /// Write a single service configuration
    fn write_service(&self, nix: &mut String, name: &str, config: &ServiceConfig, indent_level: usize) -> Result<(), String> {
        let indent = self.get_indent(indent_level);
        
        nix.push_str(&format!("{}{} = {{\n", indent, name));
        nix.push_str(&format!("{}enable = {};\n", 
            self.get_indent(indent_level + 1), 
            if config.enable { "true" } else { "false" }
        ));
        
        if !config.environment.is_empty() {
            nix.push_str(&format!("{}environment = {{\n", self.get_indent(indent_level + 1)));
            for (key, value) in &config.environment {
                nix.push_str(&format!("{}{} = \"{}\";\n", 
                    self.get_indent(indent_level + 2), 
                    key, 
                    value
                ));
            }
            nix.push_str(&format!("{}}};\n", self.get_indent(indent_level + 1)));
        }
        
        self.write_generic_settings(nix, &config.settings, indent_level + 1)?;
        
        nix.push_str(&format!("{}}};\n", indent));

        Ok(())
    }

    /// Write shell configuration
    fn write_shell_config(&self, nix: &mut String, shell: &ShellConfig) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Shell configuration\n", indent));
        }
        
        // Shell aliases
        if !shell.aliases.is_empty() {
            nix.push_str(&format!("{}home.shellAliases = {{\n", indent));
            for (alias, command) in &shell.aliases {
                nix.push_str(&format!("{}{} = \"{}\";\n", 
                    self.get_indent(2), 
                    alias, 
                    command
                ));
            }
            nix.push_str(&format!("{}}};\n\n", indent));
        }
        
        // Environment variables
        if !shell.environment.is_empty() {
            nix.push_str(&format!("{}home.sessionVariables = {{\n", indent));
            for (key, value) in &shell.environment {
                nix.push_str(&format!("{}{} = \"{}\";\n", 
                    self.get_indent(2), 
                    key, 
                    value
                ));
            }
            nix.push_str(&format!("{}}};\n\n", indent));
        }

        Ok(())
    }

    /// Write desktop configuration
    fn write_desktop_config(&self, nix: &mut String, desktop: &DesktopConfig) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Desktop environment configuration\n", indent));
        }
        
        // Theme configuration
        if let Some(theme) = &desktop.theme {
            if let Some(gtk_theme) = &theme.gtk_theme {
                nix.push_str(&format!("{}gtk.theme.name = \"{}\";\n", indent, gtk_theme));
            }
            if let Some(icon_theme) = &theme.icon_theme {
                nix.push_str(&format!("{}gtk.iconTheme.name = \"{}\";\n", indent, icon_theme));
            }
            if let Some(cursor_theme) = &theme.cursor_theme {
                nix.push_str(&format!("{}home.pointerCursor.name = \"{}\";\n", indent, cursor_theme));
            }
        }
        
        nix.push_str("\n");

        Ok(())
    }

    /// Write packages
    fn write_packages(&self, nix: &mut String, packages: &PackageSet) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Packages\n", indent));
        }
        
        nix.push_str(&format!("{}home.packages = with pkgs; [\n", indent));
        
        // Write packages by category with comments
        if !packages.system.is_empty() {
            if self.include_comments {
                nix.push_str(&format!("{}# System packages\n", self.get_indent(2)));
            }
            for pkg in &packages.system {
                nix.push_str(&format!("{}{}\n", self.get_indent(2), pkg));
            }
        }
        
        if !packages.development.is_empty() {
            if self.include_comments {
                nix.push_str(&format!("{}# Development packages\n", self.get_indent(2)));
            }
            for pkg in &packages.development {
                nix.push_str(&format!("{}{}\n", self.get_indent(2), pkg));
            }
        }
        
        if !packages.desktop.is_empty() {
            if self.include_comments {
                nix.push_str(&format!("{}# Desktop packages\n", self.get_indent(2)));
            }
            for pkg in &packages.desktop {
                nix.push_str(&format!("{}{}\n", self.get_indent(2), pkg));
            }
        }
        
        if !packages.custom.is_empty() {
            if self.include_comments {
                nix.push_str(&format!("{}# Custom packages\n", self.get_indent(2)));
            }
            for pkg in &packages.custom {
                nix.push_str(&format!("{}{}\n", self.get_indent(2), pkg));
            }
        }
        
        nix.push_str(&format!("{}];\n\n", indent));

        Ok(())
    }

    /// Write dotfiles
    fn write_dotfiles(&self, nix: &mut String, dotfiles: &[DotfileEntry]) -> Result<(), String> {
        let indent = self.get_indent(1);
        
        if self.include_comments {
            nix.push_str(&format!("{}# Dotfiles\n", indent));
        }
        
        nix.push_str(&format!("{}home.file = {{\n", indent));
        
        for entry in dotfiles {
            let target_str = entry.target.to_string_lossy();
            nix.push_str(&format!("{}\"{}\" = {{\n", self.get_indent(2), target_str));
            
            if entry.symlink {
                nix.push_str(&format!("{}source = config.lib.file.mkOutOfStoreSymlink \"{}\";\n", 
                    self.get_indent(3),
                    entry.source.display()
                ));
            } else {
                nix.push_str(&format!("{}source = \"{}\";\n", 
                    self.get_indent(3),
                    entry.source.display()
                ));
            }
            
            if let Some(mode) = &entry.mode {
                nix.push_str(&format!("{}mode = \"{}\";\n", self.get_indent(3), mode));
            }
            
            nix.push_str(&format!("{}}};\n", self.get_indent(2)));
        }
        
        nix.push_str(&format!("{}}};\n\n", indent));

        Ok(())
    }

    /// Get indentation string
    fn get_indent(&self, level: usize) -> String {
        match self.indent_style {
            IndentStyle::Spaces(n) => " ".repeat(n * level),
            IndentStyle::Tabs => "\t".repeat(level),
        }
    }

    /// Generate a flake.nix for the Home Manager configuration
    pub fn generate_flake(&self, username: &str) -> String {
        format!(r#"{{
  description = "Home Manager configuration for {}";

  inputs = {{
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    home-manager = {{
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
  }};

  outputs = {{ self, nixpkgs, home-manager, ... }}: {{
    homeConfigurations.{} = home-manager.lib.homeManagerConfiguration {{
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [ ./home.nix ];
    }};
  }};
}}"#, username, username)
    }
}
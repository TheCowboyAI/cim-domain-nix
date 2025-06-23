//! Home Manager integration demo

use std::path::Path;
use std::fs;
use tempfile::TempDir;

use cim_domain_nix::home_manager::{HomeManagerAnalyzer, ProgramConverter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Home Manager Integration Demo ===\n");

    // Create a temporary directory with sample dotfiles
    let temp_dir = TempDir::new()?;
    let dotfiles_dir = temp_dir.path();

    // Create sample .gitconfig
    let gitconfig = dotfiles_dir.join(".gitconfig");
    fs::write(&gitconfig, r#"
[user]
    name = John Doe
    email = john@example.com
[core]
    editor = vim
    autocrlf = input
[alias]
    st = status
    co = checkout
    br = branch
"#)?;

    // Create sample .vimrc
    let vimrc = dotfiles_dir.join(".vimrc");
    fs::write(&vimrc, r#"
set number
set relativenumber
syntax on
set expandtab
set tabstop=4
set shiftwidth=4

" Enable file type detection
filetype plugin indent on

" Custom keybindings
nnoremap <C-p> :Files<CR>
nnoremap <C-f> :Rg<CR>
"#)?;

    // Create sample .zshrc
    let zshrc = dotfiles_dir.join(".zshrc");
    fs::write(&zshrc, r#"
# Enable colors
autoload -U colors && colors

# Aliases
alias ll='ls -la'
alias gs='git status'
alias gp='git push'

# Environment variables
export EDITOR=vim
export PATH="$HOME/.local/bin:$PATH"

# Custom prompt
PS1="%{$fg[green]%}%n@%m%{$reset_color%}:%{$fg[blue]%}%~%{$reset_color%}$ "

# Enable history
HISTSIZE=10000
SAVEHIST=10000
HISTFILE=~/.zsh_history
"#)?;

    // Create sample .tmux.conf
    let tmux_conf = dotfiles_dir.join(".tmux.conf");
    fs::write(&tmux_conf, r#"
# Set prefix to Ctrl-a
set -g prefix C-a
unbind C-b

# Enable mouse support
set -g mouse on

# Start windows and panes at 1
set -g base-index 1
setw -g pane-base-index 1

# Split panes using | and -
bind | split-window -h
bind - split-window -v
"#)?;

    // Create a sample home.nix file
    let home_nix = dotfiles_dir.join("home.nix");
    fs::write(&home_nix, r#"
{ config, pkgs, ... }:

{
  home.username = "johndoe";
  home.homeDirectory = "/home/johndoe";
  home.stateVersion = "24.05";

  programs.git = {
    enable = true;
    userName = "John Doe";
    userEmail = "john@example.com";
  };

  programs.vim = {
    enable = true;
    settings = {
      number = true;
    };
  };

  programs.zsh = {
    enable = true;
  };

  services.gpg-agent = {
    enable = true;
  };

  home.packages = with pkgs; [
    htop
    ripgrep
    fd
  ];
}
"#)?;

    // Initialize analyzer
    let mut analyzer = HomeManagerAnalyzer::new();
    
    println!("1. Analyzing existing Home Manager configuration...");
    match analyzer.analyze_home_config(&home_nix).await {
        Ok(analysis) => {
            println!("\n   Programs configured:");
            for program in &analysis.programs {
                println!("   - {} (enabled: {}, complexity: {:?}, security: {:?})",
                    program.name,
                    program.enabled,
                    program.configuration_complexity,
                    program.security_score
                );
                if !program.dependencies.is_empty() {
                    println!("     Dependencies: {}", program.dependencies.join(", "));
                }
            }
            
            println!("\n   Services configured:");
            for service in &analysis.services {
                println!("   - {} (enabled: {})", service.name, service.enabled);
            }
            
            if !analysis.conflicts.is_empty() {
                println!("\n   ⚠️  Conflicts detected:");
                for conflict in &analysis.conflicts {
                    println!("   - {:?}: {}", conflict.conflict_type, conflict.description);
                    println!("     Affected: {}", conflict.affected_items.join(", "));
                }
            }
            
            if !analysis.suggestions.is_empty() {
                println!("\n   💡 Suggestions:");
                for suggestion in &analysis.suggestions {
                    println!("   - [{:?}] {}", suggestion.priority, suggestion.description);
                }
            }
        }
        Err(e) => {
            println!("   Error analyzing config: {}", e);
        }
    }

    println!("\n2. Converting dotfiles to Home Manager format...");
    let converter = ProgramConverter::new();
    
    for entry in fs::read_dir(dotfiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.starts_with('.') && !file_name.ends_with(".nix") {
                match converter.auto_convert(&path) {
                    Ok((program, config)) => {
                        println!("\n   ✅ Converted {}: {}", file_name, program);
                        println!("      Enabled: {}", config.enabled);
                        println!("      Settings: {} keys", config.settings.len());
                        if config.extra_config.is_some() {
                            println!("      Has extra config");
                        }
                    }
                    Err(e) => {
                        println!("\n   ❌ Failed to convert {}: {}", file_name, e);
                    }
                }
            }
        }
    }

    println!("\n3. Migrating entire dotfiles directory...");
    match analyzer.migrate_from_dotfiles(dotfiles_dir) {
        Ok(home_config) => {
            println!("\n   Migration successful!");
            println!("   - Programs: {}", home_config.programs.len());
            println!("   - Services: {}", home_config.services.len());
            println!("   - File mappings: {}", home_config.file_mappings.len());
            println!("   - Home packages: {}", home_config.home_packages.len());
            
            println!("\n   Generated Home Manager configuration:");
            println!("   programs = {{");
            for (name, config) in &home_config.programs {
                println!("     {} = {{ enable = {}; }};", name, config.enabled);
            }
            println!("   }};");
        }
        Err(e) => {
            println!("   Migration failed: {}", e);
        }
    }

    println!("\n4. Testing individual converters...");
    
    // Test Git converter
    if let Ok(git_config) = converter.convert("git", &gitconfig) {
        println!("\n   Git configuration:");
        if let Some(user_name) = git_config.settings.get("userName") {
            println!("   - User name: {:?}", user_name);
        }
        if let Some(user_email) = git_config.settings.get("userEmail") {
            println!("   - User email: {:?}", user_email);
        }
    }

    // Test supported programs
    println!("\n5. Supported programs:");
    let supported = converter.supported_programs();
    for program in supported {
        println!("   - {}", program);
    }

    println!("\n=== Demo completed successfully! ===");
    Ok(())
} 
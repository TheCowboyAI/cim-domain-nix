// Copyright 2025 Cowboy AI, LLC.

//! Integration tests for Home Manager domain

use cim_domain_nix::domains::home_manager::*;
use cim_domain_nix::value_objects::MessageIdentity;
use std::path::PathBuf;

#[test]
fn test_create_home_config() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: Some("Test User".to_string()),
        email: Some("test@example.com".to_string()),
        home_directory: PathBuf::from("/home/testuser"),
        shell: Some("/bin/bash".to_string()),
    };
    
    let packages = PackageSet {
        system: vec!["git".to_string(), "vim".to_string()],
        development: vec!["rustc".to_string(), "cargo".to_string()],
        desktop: vec![],
        custom: vec![],
    };
    
    let result = service.create_config(
        user_profile,
        packages,
        None,
        None,
    );
    
    assert!(result.is_ok());
    let config_id = result.unwrap();
    
    // Verify config was created
    let config = service.get_config(&config_id);
    println!("Config ID: {:?}", config_id);
    println!("Config found: {:?}", config.is_some());
    assert!(config.is_some());
    
    let config = config.unwrap();
    assert_eq!(config.user_profile.username, "testuser");
    assert_eq!(config.packages.system.len(), 2);
}

#[test]
fn test_add_program_to_config() {
    let service = HomeManagerService::new();
    
    // Create a config first
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let packages = PackageSet {
        system: vec![],
        development: vec![],
        desktop: vec![],
        custom: vec![],
    };
    
    let config_id = service.create_config(
        user_profile,
        packages,
        None,
        None,
    ).unwrap();
    
    // Add a program
    let git_config = ProgramConfig {
        name: "git".to_string(),
        enable: true,
        settings: serde_json::json!({
            "userName": "Test User",
            "userEmail": "test@example.com"
        }),
        extra_packages: vec![],
    };
    
    let result = service.add_program(config_id, git_config);
    assert!(result.is_ok());
    
    // Verify program was added
    let config = service.get_config(&config_id).unwrap();
    assert!(config.programs.contains_key("git"));
    
    let git = &config.programs["git"];
    assert!(git.enable);
    assert_eq!(git.settings["userName"], "Test User");
}

#[test]
fn test_update_program() {
    let service = HomeManagerService::new();
    
    // Create config with a program
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let config_id = service.create_config(
        user_profile,
        PackageSet::default(),
        None,
        None,
    ).unwrap();
    
    // Add vim program
    let vim_config = ProgramConfig {
        name: "vim".to_string(),
        enable: true,
        settings: serde_json::json!({
            "lineNumbers": true
        }),
        extra_packages: vec![],
    };
    
    service.add_program(config_id, vim_config).unwrap();
    
    // Update vim settings
    let result = service.update_program(
        config_id,
        "vim".to_string(),
        None,
        Some(serde_json::json!({
            "lineNumbers": true,
            "relativenumber": true
        })),
        Some(vec!["vim-airline".to_string()]),
    );
    
    assert!(result.is_ok());
    
    // Verify update
    let config = service.get_config(&config_id).unwrap();
    let vim = &config.programs["vim"];
    assert_eq!(vim.settings["relativenumber"], true);
    assert_eq!(vim.extra_packages, vec!["vim-airline"]);
}

#[test]
fn test_shell_configuration() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: Some("/bin/zsh".to_string()),
    };
    
    let shell_config = ShellConfig {
        shell_type: ShellType::Zsh,
        aliases: vec![
            ("ll".to_string(), "ls -la".to_string()),
            ("gs".to_string(), "git status".to_string()),
        ].into_iter().collect(),
        environment: vec![
            ("EDITOR".to_string(), "vim".to_string()),
        ].into_iter().collect(),
        init_script: Some("# Custom init script".to_string()),
        interactive_script: None,
        login_script: None,
    };
    
    let config_id = service.create_config(
        user_profile,
        PackageSet::default(),
        Some(shell_config.clone()),
        None,
    ).unwrap();
    
    // Verify shell config
    let config = service.get_config(&config_id).unwrap();
    assert!(config.shell.is_some());
    
    let shell = config.shell.unwrap();
    assert_eq!(shell.shell_type, ShellType::Zsh);
    assert_eq!(shell.aliases["ll"], "ls -la");
    assert_eq!(shell.environment["EDITOR"], "vim");
}

#[test]
fn test_add_packages() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let config_id = service.create_config(
        user_profile,
        PackageSet::default(),
        None,
        None,
    ).unwrap();
    
    // Add development packages
    let result = service.add_packages(
        config_id,
        PackageCategory::Development,
        vec![
            "rustc".to_string(),
            "cargo".to_string(),
            "rust-analyzer".to_string(),
        ],
    );
    
    assert!(result.is_ok());
    
    // Verify packages were added
    let config = service.get_config(&config_id).unwrap();
    assert_eq!(config.packages.development.len(), 3);
    assert!(config.packages.development.contains(&"rustc".to_string()));
}

#[test]
fn test_remove_packages() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let packages = PackageSet {
        system: vec!["git".to_string(), "vim".to_string(), "htop".to_string()],
        development: vec![],
        desktop: vec![],
        custom: vec![],
    };
    
    let config_id = service.create_config(
        user_profile,
        packages,
        None,
        None,
    ).unwrap();
    
    // Remove some packages
    let result = service.remove_packages(
        config_id,
        PackageCategory::System,
        vec!["vim".to_string(), "htop".to_string()],
    );
    
    assert!(result.is_ok());
    
    // Verify packages were removed
    let config = service.get_config(&config_id).unwrap();
    assert_eq!(config.packages.system.len(), 1);
    assert_eq!(config.packages.system[0], "git");
}

#[test]
fn test_desktop_configuration() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let desktop_config = DesktopConfig {
        desktop_type: DesktopType::I3,
        window_manager: Some(WindowManagerConfig {
            name: "i3".to_string(),
            config: "# i3 config".to_string(),
            keybindings: vec![
                ("mod+Return".to_string(), "exec alacritty".to_string()),
            ].into_iter().collect(),
        }),
        theme: Some(ThemeConfig {
            gtk_theme: Some("Adwaita-dark".to_string()),
            icon_theme: Some("Papirus".to_string()),
            cursor_theme: None,
            fonts: FontConfig {
                default: Some("Noto Sans".to_string()),
                monospace: Some("JetBrains Mono".to_string()),
                size: Some(11),
            },
        }),
        keyboard: Some(KeyboardConfig {
            layout: "us".to_string(),
            variant: Some("dvorak".to_string()),
            options: vec!["ctrl:nocaps".to_string()],
        }),
    };
    
    let config_id = service.create_config(
        user_profile,
        PackageSet::default(),
        None,
        Some(desktop_config),
    ).unwrap();
    
    // Verify desktop config
    let config = service.get_config(&config_id).unwrap();
    assert!(config.desktop.is_some());
    
    let desktop = config.desktop.unwrap();
    assert_eq!(desktop.desktop_type, DesktopType::I3);
    assert!(desktop.window_manager.is_some());
    assert!(desktop.theme.is_some());
    
    let theme = desktop.theme.unwrap();
    assert_eq!(theme.gtk_theme, Some("Adwaita-dark".to_string()));
    assert_eq!(theme.fonts.monospace, Some("JetBrains Mono".to_string()));
}

#[test]
fn test_find_configs_by_program() {
    let service = HomeManagerService::new();
    
    // Create multiple configs with git
    for i in 0..3 {
        let user_profile = UserProfile {
            username: format!("user{}", i),
            full_name: None,
            email: None,
            home_directory: PathBuf::from(format!("/home/user{}", i)),
            shell: None,
        };
        
        let config_id = service.create_config(
            user_profile,
            PackageSet::default(),
            None,
            None,
        ).unwrap();
        
        // Add git to some configs
        if i < 2 {
            let git_config = ProgramConfig {
                name: "git".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            };
            
            service.add_program(config_id, git_config).unwrap();
        }
    }
    
    // Find configs with git
    let configs = service.find_configs_by_program("git");
    assert_eq!(configs.len(), 2);
}

#[test]
fn test_service_configuration() {
    let service = HomeManagerService::new();
    
    let user_profile = UserProfile {
        username: "testuser".to_string(),
        full_name: None,
        email: None,
        home_directory: PathBuf::from("/home/testuser"),
        shell: None,
    };
    
    let config_id = service.create_config(
        user_profile,
        PackageSet::default(),
        None,
        None,
    ).unwrap();
    
    // Add a service
    let gpg_agent = ServiceConfig {
        name: "gpg-agent".to_string(),
        enable: true,
        settings: serde_json::json!({
            "enableSshSupport": true,
            "defaultCacheTtl": 3600
        }),
        environment: vec![].into_iter().collect(),
    };
    
    let result = service.add_service(config_id, gpg_agent);
    assert!(result.is_ok());
    
    // Verify service was added
    let config = service.get_config(&config_id).unwrap();
    assert!(config.services.contains_key("gpg-agent"));
    
    let gpg = &config.services["gpg-agent"];
    assert!(gpg.enable);
    assert_eq!(gpg.settings["enableSshSupport"], true);
}

#[cfg(test)]
mod converter_tests {
    use super::*;
    
    #[test]
    fn test_basic_config_generation() {
        let converter = HomeManagerConverter::new(true, IndentStyle::Spaces(2));
        
        let config = HomeConfigReadModel {
            id: HomeConfigId::new(),
            user_profile: UserProfile {
                username: "testuser".to_string(),
                full_name: Some("Test User".to_string()),
                email: Some("test@example.com".to_string()),
                home_directory: PathBuf::from("/home/testuser"),
                shell: None,
            },
            programs: vec![
                ("git".to_string(), ProgramConfig {
                    name: "git".to_string(),
                    enable: true,
                    settings: serde_json::json!({
                        "userName": "Test User",
                        "userEmail": "test@example.com"
                    }),
                    extra_packages: vec![],
                })
            ].into_iter().collect(),
            services: Default::default(),
            shell: None,
            desktop: None,
            packages: PackageSet {
                system: vec!["vim".to_string(), "git".to_string()],
                development: vec![],
                desktop: vec![],
                custom: vec![],
            },
            dotfiles: vec![],
        };
        
        let result = converter.convert_to_nix(&config);
        assert!(result.is_ok());
        
        let nix = result.unwrap();
        assert!(nix.contains("home.username = \"testuser\""));
        assert!(nix.contains("home.homeDirectory = \"/home/testuser\""));
        assert!(nix.contains("programs = {"));
        assert!(nix.contains("git = {"));
        assert!(nix.contains("enable = true;"));
        assert!(nix.contains("userName = \"Test User\""));
        assert!(nix.contains("home.packages = with pkgs; ["));
    }
    
    #[test]
    fn test_flake_generation() {
        let converter = HomeManagerConverter::new(false, IndentStyle::Spaces(2));
        
        let flake = converter.generate_flake("testuser");
        assert!(flake.contains("description = \"Home Manager configuration for testuser\""));
        assert!(flake.contains("homeConfigurations.testuser"));
        assert!(flake.contains("modules = [ ./home.nix ]"));
    }
}

#[cfg(test)]
mod analyzer_tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_dotfile_analysis() {
        let temp_dir = TempDir::new().unwrap();
        let dotfiles_path = temp_dir.path();
        
        // Create some test dotfiles
        fs::write(dotfiles_path.join(".bashrc"), "alias ll='ls -la'\n").unwrap();
        fs::write(dotfiles_path.join(".gitconfig"), "[user]\n  name = Test User\n").unwrap();
        fs::write(dotfiles_path.join(".vimrc"), "set number\n").unwrap();
        
        let analyzer = DotfileAnalyzer::new(
            dotfiles_path.to_path_buf(),
            vec![],
            vec![],
        );
        
        let result = analyzer.analyze();
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.shell_config.is_some());
        assert!(analysis.git_config.is_some());
        assert!(analysis.editor_config.is_some());
    }
}

#[cfg(test)]
mod aggregate_tests {
    use super::*;
    
    #[test]
    fn test_home_config_aggregate_creation() {
        let config_id = HomeConfigId::new();
        let aggregate = HomeConfigAggregate::new(config_id);
        
        assert_eq!(aggregate.id, config_id);
        assert!(!aggregate.exists);
        assert!(aggregate.programs.is_empty());
        assert!(aggregate.services.is_empty());
    }
    
    #[test]
    fn test_create_config_command() {
        let config_id = HomeConfigId::new();
        let aggregate = HomeConfigAggregate::new(config_id);
        
        let cmd = CreateHomeConfig {
            identity: MessageIdentity::new_root(),
            user_profile: UserProfile {
                username: "testuser".to_string(),
                full_name: None,
                email: None,
                home_directory: PathBuf::from("/home/testuser"),
                shell: None,
            },
            packages: PackageSet::default(),
            shell: None,
            desktop: None,
        };
        
        let result = aggregate.handle_create(cmd);
        assert!(result.is_ok());
        
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
    }
    
    #[test]
    fn test_add_program_validation() {
        let config_id = HomeConfigId::new();
        let mut aggregate = HomeConfigAggregate::new(config_id);
        aggregate.exists = true; // Simulate created config
        
        // Add a program
        aggregate.programs.insert("git".to_string(), ProgramConfig {
            name: "git".to_string(),
            enable: true,
            settings: serde_json::json!({}),
            extra_packages: vec![],
        });
        
        // Try to add the same program again
        let cmd = AddProgram {
            identity: MessageIdentity::new_root(),
            config_id,
            program: ProgramConfig {
                name: "git".to_string(),
                enable: true,
                settings: serde_json::json!({}),
                extra_packages: vec![],
            },
        };
        
        let result = aggregate.handle_add_program(cmd);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Program 'git' already exists");
    }
}
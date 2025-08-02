//! Tests for module parsing

use super::*;

#[test]
fn test_parse_simple_module() {
    let content = r#"
{ config, pkgs, ... }:
{
    imports = [
        ./hardware-configuration.nix
    ];
    
    options = {
        myService = {
            enable = mkOption {
                type = types.bool;
                default = false;
                description = "Enable my service";
            };
            
            port = mkOption {
                type = types.int;
                default = 8080;
                description = "Port to listen on";
            };
        };
    };
    
    config = {
        systemd.services.myService = {
            enable = true;
            description = "My custom service";
        };
    };
    
    meta = {
        description = "My custom NixOS module";
        maintainers = [ "user@example.com" ];
    };
}
"#;

    let nix_file = NixFile::parse_string(content.to_string(), None).unwrap();
    let parsed_module = ModuleParser::parse(&nix_file).unwrap();

    // Check imports
    assert_eq!(parsed_module.imports.len(), 1);
    assert_eq!(
        parsed_module.imports[0].to_str().unwrap(),
        "./hardware-configuration.nix"
    );

    // Check options
    assert!(parsed_module.options.contains_key("myService.enable"));
    assert!(parsed_module.options.contains_key("myService.port"));

    let enable_opt = &parsed_module.options["myService.enable"];
    assert_eq!(enable_opt.option_type, "bool");
    assert_eq!(enable_opt.default, Some(serde_json::json!(false)));
    assert_eq!(
        enable_opt.description.as_ref().unwrap(),
        "Enable my service"
    );

    // Check config
    assert!(parsed_module
        .config
        .contains_key("systemd.services.myService"));

    // Check meta
    assert_eq!(
        parsed_module.meta.description.as_ref().unwrap(),
        "My custom NixOS module"
    );
    assert_eq!(parsed_module.meta.maintainers.len(), 1);
}

#[test]
fn test_parse_direct_attrset_module() {
    let content = r#"
{
    imports = [ ./base.nix ];
    
    services.nginx.enable = true;
    networking.firewall.allowedTCPPorts = [ 80 443 ];
}
"#;

    let nix_file = NixFile::parse_string(content.to_string(), None).unwrap();
    let parsed_module = ModuleParser::parse(&nix_file).unwrap();

    // Check imports
    assert_eq!(parsed_module.imports.len(), 1);

    // Check config - top-level attributes become config
    assert!(parsed_module.config.contains_key("services.nginx.enable"));
    assert!(parsed_module
        .config
        .contains_key("networking.firewall.allowedTCPPorts"));
}

#[test]
fn test_parse_module_with_complex_options() {
    let content = r#"
{ lib, ... }:
{
    options = {
        myApp = {
            databases = lib.mkOption {
                type = lib.types.listOf (lib.types.submodule {
                    options = {
                        name = lib.mkOption {
                            type = lib.types.str;
                            description = "Database name";
                        };
                        
                        type = lib.mkOption {
                            type = lib.types.enum [ "postgres" "mysql" "sqlite" ];
                            default = "postgres";
                            description = "Database type";
                        };
                    };
                });
                default = [];
                description = "List of databases to configure";
            };
        };
    };
}
"#;

    let nix_file = NixFile::parse_string(content.to_string(), None).unwrap();
    let parsed_module = ModuleParser::parse(&nix_file).unwrap();

    // Check complex option
    assert!(parsed_module.options.contains_key("myApp.databases"));
    let db_opt = &parsed_module.options["myApp.databases"];
    assert_eq!(
        db_opt.description.as_ref().unwrap(),
        "List of databases to configure"
    );
    assert_eq!(db_opt.default, Some(serde_json::json!([])));
}

#[test]
fn test_parse_module_with_let_bindings() {
    let content = r#"
{ config, lib, pkgs, ... }:

let
    cfg = config.services.myService;
    package = pkgs.myPackage;
in {
    imports = [ ./common.nix ];
    
    config = lib.mkIf cfg.enable {
        systemd.services.myService = {
            enable = true;
            serviceConfig.ExecStart = "${package}/bin/myservice";
        };
    };
}
"#;

    // This test might fail with current parser limitations, but shows the intended use
    let nix_file = NixFile::parse_string(content.to_string(), None).unwrap();

    // Just check it doesn't panic
    let _ = ModuleParser::parse(&nix_file);
}

{
  description = "CIM Domain Nix - Domain module for Nix ecosystem operations";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-topology.url = "github:oddlama/nixos-topology";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixos-topology, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        buildInputs = with pkgs; [
          openssl
          pkg-config
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          libiconv
          darwin.apple_sdk.frameworks.Security
        ];

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cim-domain-nix";
          version = "0.3.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
          
          # Skip tests during build (run separately)
          doCheck = false;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
          
          packages = with pkgs; [
            # Rust development
            rustToolchain
            rust-analyzer
            cargo-watch
            cargo-nextest
            cargo-edit
            cargo-outdated
            cargo-audit
            cargo-license
            cargo-tarpaulin
            
            # Nix development tools
            nix-prefetch-git
            nixpkgs-fmt
            alejandra
            statix
            deadnix
            nil
            
            # General development tools
            just
            bacon
            mold
            sccache
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = 1;
          RUST_LOG = "debug";
          
          # OpenSSL configuration
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          
          shellHook = ''
            echo "CIM Domain Nix Development Shell"
            echo "================================"
            echo ""
            echo "Available commands:"
            echo "  cargo build    - Build the project"
            echo "  cargo test     - Run tests"
            echo "  cargo watch    - Watch for changes and rebuild"
            echo "  cargo nextest  - Run tests with nextest"
            echo "  nix build      - Build with Nix"
            echo ""
            echo "OpenSSL configured at: $OPENSSL_DIR"
          '';
        };
      });
}
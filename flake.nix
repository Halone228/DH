{
  description = "dayhelper — Telegram reminder + anti-procrastination bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.sqlx-cli
            pkgs.sqlite
            pkgs.openssl
            pkgs.pkg-config
          ];

          shellHook = ''
            export DATABASE_URL="sqlite://$PWD/dayhelper.db"
            export RUST_LOG=''${RUST_LOG:-info,sqlx=warn}
          '';
        };
      });
}

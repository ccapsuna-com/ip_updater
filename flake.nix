{
  description = "IP updater binary build environment";

   inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05"; # Or a specific stable channel like "nixos-24.05"
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Apply the rust-overlay to nixpkgs to get access to custom Rust toolchains
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Select the latest stable Rust toolchain
        # You can choose other channels like `stable.latest`, `nightly`, or specific versions like `rust-bin.toolchains.1_77_0`
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src" # Required for rust-analyzer and some IDE features
            "clippy"
            "rustfmt"
          ];
          # Add any specific targets you need, e.g., "wasm32-unknown-unknown"
          # targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          name = "rust-dev-shell";
          # # The below was tricky to find but very important for VS Code
          # # if you run without an extension and launch from within the dev shell
          # buildInputs = [
          #   pkgs.bashInteractive
          # ];
          # Packages available in the development shell
          packages = with pkgs; [
            rustToolchain
            rust-analyzer
          ];

          # Environment variables


          # Environment variables or shell hooks to run when entering the shell
          shellHook = ''
            echo "Entering Rust development environment!"

            # Set RUST_SRC_PATH for rust-analyzer
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

            # Inform user about common commands
            echo "You can now use:"
            echo "  cargo build/run/test"
            echo "  rustc --version"
          '';

        };
      });
}
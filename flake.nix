{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rust_toolchain =
          p: pkgs.rust-bin.stable.latest;
        rust_toolchain_wasm = p: (rust_toolchain p).default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain rust_toolchain_wasm;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ ];
      in
      rec {
        packages = {
          worker-wasm = craneLib.buildPackage {
            inherit nativeBuildInputs buildInputs;
            src = ./.;
            cargoExtraArgs = "--target wasm32-unknown-unknown";
            doCheck = false;
            CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          };

          worker = pkgs.stdenv.mkDerivation {
            name = "cloudfree-worker";
            src = ./.;

            buildPhase = ''
              # Just copy source files - build happens at deploy time
              mkdir -p worker
              cp -r src Cargo.toml Cargo.lock wrangler.toml worker/

              # Copy .cargo if it exists
              if [ -d .cargo ]; then
                cp -r .cargo worker/
              fi
            '';

            installPhase = ''
              mkdir -p $out/worker
              cp -r worker/* $out/worker/
            '';
          };
        };

        defaultPackage = self.packages.${system}.worker;

        devShell = pkgs.mkShell {
          inherit buildInputs;
          nativeBuildInputs = nativeBuildInputs ++ [
            pkgs.wrangler
            ((rust_toolchain pkgs).default.override {
              extensions = [ "rust-src" "rustfmt" "rust-analyzer" "clippy" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            (pkgs.python3.withPackages (ps: with ps; [ requests ]))
          ];
        };
      }
    );
}

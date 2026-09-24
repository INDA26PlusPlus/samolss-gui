# Heavily inspired by:
# https://github.com/DeterminateSystems/zero-to-nix/blob/main/nix/templates/pkg/rust/flake.nix
# and
# https://github.com/numtide/treefmt-nix
{

  description = "Simple flake";

  inputs = {
    nixpkgs.url = "github:Nixos/nixpkgs/nixpkgs-unstable";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      rust-overlay,
      treefmt-nix,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            pkgs = import nixpkgs {
              inherit system;

              overlays = [
                rust-overlay.overlays.default
                self.overlays.default
              ];
            };

          }
        );

      treefmtEval = forAllSystems (
        { pkgs }:
        treefmt-nix.lib.evalModule pkgs ./treefmt.nix
      );
    in
    {
      overlays.default = final: prev: {
        rustToolchain = final.rust-bin.stable.latest.default;
      };

      formatter = forAllSystems (
        { pkgs }:
        treefmtEval.${pkgs.system}.config.build.wrapper
      );

      checks = forAllSystems (
        { pkgs }: {
          formatter = treefmtEval.${pkgs.system}.config.build.check self;
        }
      );

      # Taken from https://discourse.nixos.org/t/how-to-provide-alsa-pc-to-environment/54670/5
      packages = forAllSystems (
        { pkgs }: {
          default =
            let
              rustPlatform = pkgs.makeRustPlatform {
                cargo = pkgs.rust-bin.stable.latest.minimal;
                rustc = pkgs.rust-bin.stable.latest.minimal;
              };
            in
            rustPlatform.buildRustPackage rec {
              name = "samolss-gui";
              src = self;
              nativeBuildInputs = with pkgs; [ pkg-config ];
              buildInputs = with pkgs; [
                alsa-lib.dev
                udev.dev
                xorg.libX11
                xorg.libXrandr
                xorg.libXcursor
                xorg.libxcb
                xorg.libXi
                wayland
                libxkbcommon
                libxkbcommon.dev
                vulkan-loader
                vulkan-tools
                glfw
              ];
              runtimeLibs = with pkgs; [
                udev.dev
                xorg.libX11
                xorg.libXrandr
                xorg.libXcursor
                xorg.libxcb
                xorg.libXi
                wayland
                libxkbcommon
                libxkbcommon.dev
                vulkan-loader
                vulkan-tools
              ];
              postFixup = ''
                patchelf --add-rpath ${pkgs.lib.makeLibraryPath runtimeLibs} $out/bin/*
                mkdir $out/bin/resources
                cp -r assets $out/bin/resources/assets
              '';
              # postInstall = ''
              #   mkdir $out/bin/resources
              #   cp -r assets $out/bin/resources/assets
              # '';
              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "chess_library-0.1.0" = "sha256-vGHkDIXncz1gGDAzj5wY/XQWo+DtIYgx6gm/dcWQnag=";
                };
              };
              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
            };
        }
      );

      devShells = forAllSystems (
        { pkgs }: {
          default = pkgs.mkShell {
            packages = [
              (pkgs.rustToolchain.override {
                extensions = [
                  "rust-src"
                  "rust-analyzer"
                  "clippy"
                ];
              })
            ];
          };
        }
      );

    };
}

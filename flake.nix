{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    devenv = {
      url = "github:cachix/devenv";
      inputs = {
        cachix.follows = "cachix";
        flake-compat.follows = "flake-compat";
        nixpkgs.follows = "nixpkgs";
        pre-commit-hooks.follows = "pre-commit-hooks";
      };
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    cachix = {
      url = "github:cachix/cachix";
      inputs = {
        devenv.follows = "devenv";
        flake-compat.follows = "flake-compat";
        nixpkgs.follows = "nixpkgs";
        pre-commit-hooks.follows = "pre-commit-hooks";
      };
    };

    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs = {
        flake-compat.follows = "flake-compat";
        nixpkgs.follows = "nixpkgs";
        nixpkgs-stable.follows = "nixpkgs";
      };
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { devenv, fenix, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        {
          packages = {
            docker = pkgs.dockerTools.buildLayeredImage {
              name = "rika";
              tag = "latest";
              config.Cmd =
                let
                  toolchain =
                    with fenix.packages.${system};
                    combine [
                      unstable.cargo
                      unstable.rustc
                    ];
                in
                "${
                  (pkgs.makeRustPlatform {
                    cargo = toolchain;
                    rustc = toolchain;
                  }).buildRustPackage
                    {
                      name = "rika";
                      src = ./.;
                      cargoLock.lockFile = ./Cargo.lock;
                    }
                }/bin/rika";
            };
          };

          devShells.default = devenv.lib.mkShell {
            inherit inputs pkgs;
            modules = [
              {
                packages = with pkgs; [
                  openssl
                  sqlx-cli
                  nixfmt-rfc-style
                ];

                languages.rust = {
                  enable = true;
                  channel = "nightly";
                };
              }
            ];
          };
        };
    };
}

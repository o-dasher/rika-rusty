{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { fenix, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        let
          toolchain = fenix.packages.${system}.latest;
        in
        {
          packages = {
            docker = pkgs.dockerTools.buildLayeredImage {
              name = "rika";
              tag = "latest";
              config.Cmd = "${
                (pkgs.makeRustPlatform {
                  cargo = toolchain.cargo;
                  rustc = toolchain.rustc;
                }).buildRustPackage
                  {
                    name = "rika";
                    src = ./.;
                    cargoLock.lockFile = ./Cargo.lock;
                  }
              }/bin/rika";
            };
          };

          devShells.default = pkgs.mkShell {
            packages =
              (with pkgs; [
                openssl
                pkg-config
                sqlx-cli
                nixfmt-rfc-style
              ])
              ++ (with toolchain; [
                cargo
                clippy
                rustc
                rustfmt
                rust-analyzer
              ]);
          };
        };
    };
}

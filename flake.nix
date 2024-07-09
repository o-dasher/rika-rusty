{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-filter.url = "github:numtide/nix-filter";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      fenix,
      crane,
      nix-filter,
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        {
          pkgs,
          system,
          lib,
          ...
        }:
        let
          commonEnvironment = {
            SQLX_OFFLINE = "true";
          };

          toolchain = fenix.packages.${system}.complete;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain.toolchain;

          buildInputs = with pkgs; [ openssl ];
          nativeBuildInputs = with pkgs; [ pkg-config ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        in
        {
          packages =
            let
              pkgName = "osaka";
              pkg =
                (craneLib.buildPackage (
                  {
                    buildInputs = buildInputs;
                    nativeBuildInputs = nativeBuildInputs ++ [ pkgs.makeWrapper ];

                    src = nix-filter.lib {
                      root = ./.;
                      include = [
                        "Cargo.toml"
                        "Cargo.lock"
                        "src"
                        "migrations"
                        ".sqlx"
                      ];
                    };

                    postInstall = ''
                      wrapProgram $out/bin/${pkgName} \
                          --prefix LD_LIBRARY_PATH : "${LD_LIBRARY_PATH}"
                    '';
                  }
                  // commonEnvironment
                ))
                // {
                  meta.mainProgram = pkgName;
                };
            in
            {
              default = pkg;
              docker = pkgs.dockerTools.buildLayeredImage {
                tag = "latest";
                name = pkgName;
                contents = with pkgs; [
                  cacert
                  bash # required to deploy on heroku
                ];
                config.Cmd = [ (lib.getExe pkg) ];
              };
            };

          devShells =
            let
              commonPackages =
                (with toolchain; [
                  clippy
                  rustfmt
                  rustc
                  cargo
                ])
                ++ buildInputs;
            in
            {
              ci = pkgs.mkShell { packages = commonPackages; };
              default = pkgs.mkShell (
                {
                  packages =
                    (with toolchain; [ rust-analyzer ])
                    ++ (with pkgs; [
                      nixfmt-rfc-style
                      sqlx-cli
                    ])
                    ++ nativeBuildInputs
                    ++ commonPackages;
                }
                // commonEnvironment
              );
            };
        };
    };
}

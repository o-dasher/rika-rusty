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

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { fenix, crane, ... }@inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { pkgs, system, ... }:
        let
          DATABASE_URL = "postgres://rika:rika@localhost:5432/rika";
          toolchain = fenix.packages.${system}.complete;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain.toolchain;
          buildDeps = with pkgs; [
            pkg-config
            openssl
            sqlx-cli
          ];
        in
        {
          packages.default =
            let
              src = ./.;
              pkg = craneLib.buildPackage {
                src = craneLib.cleanCargoSource src;
                buildInputs = buildDeps ++ (with pkgs; [ docker-compose ]);

                DATABASE_URL = DATABASE_URL;

                preBuild = ''
                  docker-compose -f ${src + /docker-compose.yaml} up -d
                  sqlx database create
                  sqlx migrate --source ${src + /migrations} run
                  docker-compose down
                '';
              };
              pkgName = "rika";
            in
            pkgs.dockerTools.buildLayeredImage {
              name = pkgName;
              tag = "latest";
              config.Cmd = "${pkg}/bin/${pkgName}";
              config.Expose = "5432";
            };

          devShells.default = pkgs.mkShell {
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.openssl ];
            DATABASE_URL = DATABASE_URL;

            packages =
              (with pkgs; [ nixfmt-rfc-style ])
              ++ (with toolchain; [
                clippy
                rustfmt
                rust-analyzer
              ])
              ++ buildDeps;
          };
        };
    };
}

{ pkgs, ... }:
let
  name = "osaka";
in
{
  packages = with pkgs; [
    openssl
    sqlx-cli
    nixfmt-rfc-style
  ];

  env.DATABASE_URL = "postgres:///${name}";

  dotenv.enable = true;
  languages.rust = {
    enable = true;
    channel = "nightly";
  };
  services.postgres = {
    enable = true;
    initialDatabases = [ { name = name; } ];
  };
}

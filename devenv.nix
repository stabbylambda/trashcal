{ pkgs, lib, config, inputs, ... }:

{
  dotenv.disableHint = true;

  env.CHRONO_TZ_TIMEZONE_FILTER = "(UTC|US/.*)";

  languages.rust.enable = true;
  languages.rust.channel = "stable";
  languages.rust.targets = [
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
  ];

  languages.javascript.enable = true;
  languages.javascript.package = pkgs.nodejs_24;
  languages.javascript.pnpm.enable = true;

  packages = [
    pkgs.rustup # not actually using rustup, but the cdk builder expects it
    pkgs.cargo-zigbuild
    pkgs.cargo-lambda
    pkgs.cargo-watch
  ];
}

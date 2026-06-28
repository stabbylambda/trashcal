{ pkgs, lib, config, inputs, ... }:

let
  rustTargets = [
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
  ];

  # The @cdklabs/aws-lambda-rust CDK construct decides between local
  # (cargo-zigbuild) and Docker bundling by probing
  # `rustup target list --installed`. Our Rust toolchain comes from the Nix
  # rust-overlay rather than rustup, so the real rustup has no default toolchain
  # and the probe fails, forcing (unavailable) Docker bundling. This shim reports
  # the configured targets as installed so the construct takes the Docker-free
  # local build path via cargo-zigbuild.
  rustupShim = pkgs.writeShellScriptBin "rustup" ''
    if [ "$1" = "target" ] && [ "$2" = "list" ]; then
      printf '%s\n' ${lib.escapeShellArgs rustTargets}
      exit 0
    fi
    exit 0
  '';
in
{
  dotenv.disableHint = true;

  env.CHRONO_TZ_TIMEZONE_FILTER = "(UTC|US/.*)";

  languages.rust.enable = true;
  languages.rust.channel = "stable";
  languages.rust.targets = rustTargets;

  languages.javascript.enable = true;
  languages.javascript.package = pkgs.nodejs_24;
  languages.javascript.corepack.enable = true;

  packages = [
    rustupShim # rustup shim so the cdk rust builder bundles locally instead of via Docker
    pkgs.cargo-zigbuild
    pkgs.cargo-lambda
    pkgs.cargo-watch
  ];
}

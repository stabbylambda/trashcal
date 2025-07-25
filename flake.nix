{
  inputs = {
    nixpkgs.url = "github:cachix/devenv-nixpkgs/rolling";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url =  "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;
      });

      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  env.CHRONO_TZ_TIMEZONE_FILTER = "(UTC|US/.*)";

                  aws-vault.enable=true;
                  aws-vault.profile="trashcal";
                  aws-vault.awscliWrapper.enable=true;

                  languages.rust.enable = true;
                  languages.rust.channel = "stable";
                  languages.rust.targets = [
                    "x86_64-unknown-linux-gnu"
                    "aarch64-unknown-linux-gnu"
                  ];

                  languages.javascript.enable = true;
                  languages.javascript.package = pkgs.nodejs_22;
                  languages.javascript.pnpm.enable = true;

                  packages = [
                    pkgs.aws-vault
                    pkgs.rustup # not actually using rustup, but the cdk builder expects it
                    pkgs.cargo-zigbuild
                    pkgs.cargo-lambda
                    pkgs.cargo-watch
                  ];
                }
                {
                  packages = nixpkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs.darwin.apple_sdk; [
                    frameworks.Security
                    frameworks.SystemConfiguration
                  ]);
                }
              ];
            };
          });
    };
}

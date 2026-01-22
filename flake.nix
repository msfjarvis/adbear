{
  description = "package definition and devshell for adbear";

  inputs.nixpkgs.url = "github:msfjarvis/nixpkgs/nixpkgs-unstable";

  inputs.systems.url = "github:msfjarvis/flake-systems";

  inputs.advisory-db.url = "github:rustsec/advisory-db";
  inputs.advisory-db.flake = false;

  inputs.crane.url = "github:ipetkov/crane";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";

  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.flake-utils.inputs.systems.follows = "systems";

  inputs.flake-compat.url = "git+https://git.lix.systems/lix-project/flake-compat";
  inputs.flake-compat.flake = false;

  outputs =
    {
      nixpkgs,
      advisory-db,
      crane,
      devshell,
      fenix,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        inherit (pkgs) lib stdenv;

        rustStable = (import fenix { inherit pkgs; }).fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-vra6TkHITpwRyA5oBKAHSX0Mi6CBDNQD+ryPSpxFsfg=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = [ pkgs.openssl ];
          nativeBuildInputs = [ pkgs.pkg-config ] ++ lib.optionals stdenv.isDarwin [ pkgs.libiconv ];
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // { doCheck = false; });

        adbear = craneLib.buildPackage (commonArgs // { doCheck = false; });
        adbear-clippy = craneLib.cargoClippy (commonArgs // { inherit cargoArtifacts; });
        adbear-fmt = craneLib.cargoFmt (commonArgs // { });
        adbear-audit = craneLib.cargoAudit (commonArgs // { inherit advisory-db; });
      in
      # adbear-nextest = craneLib.cargoNextest (
      #   commonArgs
      #   // {
      #     inherit cargoArtifacts;
      #     src = ./.;
      #     partitions = 1;
      #     partitionType = "count";
      #   }
      # );
      {
        checks = {
          inherit
            adbear
            adbear-audit
            adbear-clippy
            adbear-fmt
            # adbear-nextest
            ;
        };

        packages.default = adbear;

        apps.default = flake-utils.lib.mkApp { drv = adbear; };

        devShells.default = pkgs.devshell.mkShell {
          bash = {
            interactive = "";
          };

          env = [
            {
              name = "DEVSHELL_NO_MOTD";
              value = 1;
            }
          ];

          packages = with pkgs; [
            bacon
            cargo-dist
            cargo-nextest
            cargo-release
            rustStable
            stdenv.cc
          ];
        };
      }
    );
}

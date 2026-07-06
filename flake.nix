{
  description = "signal-spirit - Signal contract for the ordinary spirit surface";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-build = {
      url = "github:LiGoldragon/rust-build";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-build }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        rust = rust-build.lib.${system}.fromPkgs pkgs;
        inherit (rust) craneLib toolchain;
        examplesFilter = path: _type: builtins.match ".*/examples(/.*)?$" path != null;
        schemaFilter = path: type:
          type == "regular" &&
          (pkgs.lib.hasSuffix ".nota" path || pkgs.lib.hasSuffix ".schema" path);
        src = rust.cleanSource {
          root = ./.;
          extraFilters = [
            examplesFilter
            schemaFilter
          ];
        };
        cargoVendorDirectory = craneLib.vendorCargoDeps { inherit src; };
        commonArguments = {
          inherit src cargoVendorDirectory;
          strictDeps = true;
        };
        notaTextArguments = commonArguments // {
          cargoExtraArgs = "--features nota-text";
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArguments;
        notaTextCargoArtifacts = craneLib.buildDepsOnly notaTextArguments;
      in
      {
        packages.default = craneLib.buildPackage (commonArguments // { inherit cargoArtifacts; });
        checks = {
          build = craneLib.cargoBuild (commonArguments // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArguments // { inherit cargoArtifacts; });
          test-nota-text = craneLib.cargoTest (notaTextArguments // {
            cargoArtifacts = notaTextCargoArtifacts;
          });
          doc = craneLib.cargoDoc (commonArguments // {
            inherit cargoArtifacts;
            RUSTDOCFLAGS = "-D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          clippy = craneLib.cargoClippy (commonArguments // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
        };
        devShells.default = pkgs.mkShell {
          name = "signal-spirit";
          packages = [ pkgs.jujutsu pkgs.pkg-config toolchain ];
        };
      });
}

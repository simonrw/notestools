{pkgs ? import <nixpkgs> {}}:
with pkgs;
  rustPlatform.buildRustPackage {
    pname = "notes";
    version = "0.1.0";

    src = ./.;

    cargoLock.lockFile = ./Cargo.lock;
  }

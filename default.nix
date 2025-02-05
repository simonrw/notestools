{pkgs ? import <nixpkgs> {}}:
with pkgs;
  rustPlatform.buildRustPackage {
    pname = "notes";
    version = "0.1.0";

    src = ./.;

    cargoLock.lockFile = ./Cargo.lock;

    nativeBuildInputs = [
      pkgs.installShellFiles
    ];

    postInstall = ''
    installShellCompletion \
      --cmd notes \
      --bash <($out/bin/notes completion --shell bash) \
      --fish <($out/bin/notes completion --shell fish) \
      --zsh <($out/bin/notes completion --shell zsh)
    '';
  }

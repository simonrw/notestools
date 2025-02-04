{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell rec {
    packages =
      [
        hyperfine
        rustup
        clang
      ]
      ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
        libiconv
      ])
      ++ lib.optionals stdenv.isLinux [
        mold
      ];

    env = {
      RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
      LD_LIBRARY_PATH = lib.makeLibraryPath packages;
    };
  }

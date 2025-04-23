{
  description = "Flake to build and develop Nyxi";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

        # Setting variables that everything uses
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        CLANG_PATH = "${pkgs.llvmPackages.clang}/bin/clang";

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          inherit LIBCLANG_PATH CLANG_PATH;
          buildInputs = with pkgs; [ pkg-config openssl ];
          nativeBuildInputs = with pkgs; [ util-linux.dev ];
          src = ./.;
        };

        check = naersk'.buildPackage {
          src = ./.;
          mode = "check";
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          inherit LIBCLANG_PATH CLANG_PATH;
          buildInputs = with pkgs; [ pkg-config openssl ];
          nativeBuildInputs = with pkgs; [ rustc cargo util-linux.dev ];
        };
      }
    );
}

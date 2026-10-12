{
  description = "Aurora";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05-small";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    naersk,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
    };

    rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
      extensions = ["rust-src" "rust-analyzer" "clippy" "rustfmt"];
    };

    naerskLib = pkgs.callPackage naersk {
      rustc = rustToolchain;
      cargo = rustToolchain;
    };
  in {
    packages.${system}.default = naerskLib.buildPackage {
      src = ./.;
      pname = "aurora";

      buildInputs = [pkgs.systemd.dev];
      nativeBuildInputs = [pkgs.pkg-config];

      postInstall = ''
        install -Dm644 packaging/udev/70-aurora.rules \
          $out/lib/udev/rules.d/70-aurora.rules
      '';
    };

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        rustToolchain
        pkgs.usbutils # lsusb
      ];

      buildInputs = [pkgs.systemd.dev];
      nativeBuildInputs = [pkgs.pkg-config];
    };

    formatter.${system} = pkgs.alejandra;
  };
}

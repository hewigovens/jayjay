{
  description = "JayJay GPUI shell — Linux/Windows native frontend for jj.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    nix-appimage = {
      url = "github:ralismark/nix-appimage";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  # Linux only — the SwiftUI shell handles macOS, and bundling for Windows
  # is a separate effort (no AppImage there).
  outputs = { self, nixpkgs, flake-utils, crane, nix-appimage }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        # Runtime libs GPUI dlopens (Vulkan, Wayland, X11) and crates link to.
        runtimeDeps = with pkgs; [
          fontconfig
          wayland
          libxkbcommon
          xorg.libxcb
          xorg.libX11
          vulkan-loader
          openssl
          zstd
        ];

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          cargoExtraArgs = "-p jayjay-gpui";

          nativeBuildInputs = with pkgs; [ pkg-config clang ];
          buildInputs = runtimeDeps;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        jayjay-gpui = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = "jayjay-gpui";

          # Bake runtime lib paths into the binary so the AppImage finds Vulkan
          # / Wayland / X11 after the closure is packaged.
          postFixup = ''
            patchelf --set-rpath "${pkgs.lib.makeLibraryPath runtimeDeps}" \
              $out/bin/jayjay-gpui || true
          '';
        });
      in
      {
        packages = {
          inherit jayjay-gpui;
          default = jayjay-gpui;
          appimage = nix-appimage.bundlers.${system}.default jayjay-gpui;
        };

        apps.default = flake-utils.lib.mkApp { drv = jayjay-gpui; };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ jayjay-gpui ];
          packages = with pkgs; [ rustc cargo clippy rustfmt ];
        };
      }
    );
}

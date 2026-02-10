{pkgs, ...}: {
  config = {
    devShell = {
      contents = with pkgs; [
        cargo-release
      ];
    };
    programs.rust = {
      enable = true;
      toolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      };
    };
  };
}

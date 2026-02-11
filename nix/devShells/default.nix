{pkgs, ...}: {
  config = {
    devShell = {
      contents = with pkgs; [
        cargo-release
        cargo-lambda
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

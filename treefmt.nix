# treefmt-nix module, see https://github.com/numtide/treefmt-nix
{ ... }:
{
  projectRootFile = "flake.nix";
  programs.nixfmt.enable = true;
  programs.rustfmt.enable = true;
  programs.mdformat.enable = true;
}

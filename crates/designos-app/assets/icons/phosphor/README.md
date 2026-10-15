# Vendored Phosphor Icons (`https://phosphoricons.com/`)

Icon set: [Phosphor Icons](https://phosphoricons.com/) (`@phosphor-icons/core@2.1.1`, MIT license, see `LICENSE`).
Weight: `regular` (`viewBox="0 0 256 256"`).

Every file in this directory is embedded into the binary via `include_str!` in
`src/icons.rs`, rasterised with `resvg` into a single-channel alpha mask at the
exact device pixel size it is drawn at, and tinted by egui's vertex colour
when painting. `Icon::phosphor_name` maps every UI icon to the file here; a unit
test fails if an icon is unmapped or if a vendored SVG is unused.

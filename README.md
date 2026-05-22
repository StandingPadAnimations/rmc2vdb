# rmc2VDB

A Rust utility to export biome data from Minecraft world to VDB grids.

This project is still a WIP, expect things to break! Not for production use!

## Usage

For use in Blender

```bash
cargo run --release -- \
  --world "/path/to/world" \
  --output "world.vdb" \
  --start -546,-64,656 \
  --end -365,319,826 \
  --remap "X -Z Y" \
```

If you have an OBJ with a CommonMCOBJ header, you can also pass that in
directly, assuming the source Minecraft world exists on the machine:

```bash
cargo run --release -- \
  --commonmcobj-source "path/to/export.obj"
  --remap "X -Z Y" \
```

This will output the VDB with the same name as the input OBJ, just with
the VDB file extension.

## VDB Structure

The exported file contains six grids:

- `density` (Float): 1.0 for solid blocks, 0.0 for air.
- `color` (Vec3f): The sRGB tint of the block (0.0 - 1.0).
- `block_index` (Int32): Integer corresponding to block.
- `biome_index` (Int32): Integer corresponding to biome.
- `temperature` (Float): Temperature of the biome.
- `downfall` (Float): Downfall of the biome.

### Metadata

The `density` grid contains a dictionary of names mapping to the indices:

- `block_name_N`: The Minecraft ID for a given block index, where `N` is the index.
- `biome_name_N`: The Minecraft ID for a given biome index, where `N` is the index.

## Build Requirements

- Rust 1.70+
- `libopenvdb`
- `libtbb`
- `Imath` (OpenEXR)
- C++17 compatible compiler

For the most part, `cargo build` should be enough once the dependencies
are present. At the moment, it's expected for these to be installed by
the system.

This has only been tested on Linux. Here be dragons!

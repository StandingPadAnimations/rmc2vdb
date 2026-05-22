/*
 * Copyright (C) 2026 Maryam Sheikh (Mahid Sheikh) <mahid@standingpad.org>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

mod commonmcobj_parser;
mod tint;
mod world;

use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::commonmcobj_parser::parse_header;
use crate::tint::{DEFAULT_BIOME_INFO, get_biome_map, get_tint};
use crate::world::{World, is_transparent};

#[cxx::bridge]
mod ffi {
    /// Voxel data point passed to the C++ VDB bridge.
    struct VdbPoint {
        x: f32,
        y: f32,
        z: f32,
        block: String,
        biome: String,
        temperature: f32,
        downfall: f32,
        r: f32,
        g: f32,
        b: f32,
    }

    unsafe extern "C++" {
        include!("rmc2vdb/src/vdb_writer.hpp");
        /// Serializes collected points into an OpenVDB voxel grid file.
        fn write_vdb(filename: &str, points: &[VdbPoint]) -> Result<()>;
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Minecraft world directory (contains 'region').
    #[arg(short, long)]
    world: Option<PathBuf>,

    /// Output path for the .vdb volume.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Start coordinates (x,y,z).
    #[arg(
        short,
        long,
        num_args = 3,
        value_delimiter = ',',
        allow_negative_numbers = true
    )]
    start: Vec<i32>,

    /// End coordinates (x,y,z).
    #[arg(
        short,
        long,
        num_args = 3,
        value_delimiter = ',',
        allow_negative_numbers = true
    )]
    end: Vec<i32>,

    /// Export all non-air blocks (solid volume).
    #[arg(long, default_value_t = false)]
    solid: bool,

    /// Remap axes, e.g. "X -Z Y" for Blender.
    #[arg(long, default_value = "X Y Z")]
    remap: String,

    /// Absolute offset added to all coordinates (x,y,z).
    #[arg(
        long,
        num_args = 3,
        value_delimiter = ',',
        allow_negative_numbers = true,
        default_value = "0,0,0"
    )]
    offset: Vec<f32>,

    #[arg(long)]
    commonmcobj_source: Option<PathBuf>,
}

/// Orchestrates coordinate transformation and translation.
struct CoordinateMapper {
    axes: Vec<String>,
}

impl CoordinateMapper {
    /// Initializes mapper from the remap string and offset vector.
    fn new(remap_str: &str) -> Result<Self> {
        let axes: Vec<String> = remap_str
            .split_whitespace()
            .map(|s| s.to_uppercase())
            .collect();

        if axes.len() != 3 {
            anyhow::bail!("--remap must have 3 parts, e.g., 'X -Z Y'");
        }

        Ok(Self { axes })
    }

    /// Transforms integer Minecraft coordinates to output space.
    fn map(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let get_val = |axis: &str| -> f32 {
            match axis {
                "X" => x,
                "-X" => -x,
                "Y" => y,
                "-Y" => -y,
                "Z" => z,
                "-Z" => -z,
                _ => 0.0,
            }
        };

        (
            get_val(&self.axes[0]),
            get_val(&self.axes[1]),
            get_val(&self.axes[2]),
        )
    }
}

fn convert_world(
    min_bounds: (i32, i32, i32),
    max_bounds: (i32, i32, i32),
    world: PathBuf,
    output: PathBuf,
    remap: &str,
    solid: bool,
    offset: (f32, f32, f32),
) -> Result<()> {
    let mut world = World::new(&world);
    let coord_mapper = CoordinateMapper::new(remap)?;

    let (min_x, min_y, min_z) = min_bounds;
    let (max_x, max_y, max_z) = max_bounds;

    let total_voxels =
        (max_x - min_x + 1) as u64 * (max_y - min_y + 1) as u64 * (max_z - min_z + 1) as u64;
    let mut last_percent = 0;

    let mut points = Vec::new();

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            for z in min_z..=max_z {
                let current_voxel =
                    (x - min_x) as u64 * (max_y - min_y + 1) as u64 * (max_z - min_z + 1) as u64
                        + (y - min_y) as u64 * (max_z - min_z + 1) as u64
                        + (z - min_z) as u64;

                let percent = (current_voxel * 100 / total_voxels.max(1)) as u32;
                if percent >= last_percent + 5 {
                    println!("Progress: {}%", percent);
                    last_percent = percent;
                }

                let (block, biome) = world.get_block_and_biome(x, y, z)?;
                if is_transparent(&block) {
                    continue;
                }

                if !solid && !is_block_visible(&mut world, x, y, z)? {
                    continue;
                }

                let (r, g, b) = get_tint(&block, z.max(0) as f32, &biome);
                let (out_x, out_y, out_z) = coord_mapper.map(
                    x as f32 + offset.0,
                    y as f32 + offset.1,
                    z as f32 + offset.2,
                );

                let info = get_biome_map()
                    .get(&biome as &str)
                    .unwrap_or(&DEFAULT_BIOME_INFO);

                points.push(ffi::VdbPoint {
                    x: out_x,
                    y: out_y,
                    z: out_z,
                    block,
                    biome,
                    temperature: info.temperature,
                    downfall: info.downfall,
                    r,
                    g,
                    b,
                });
            }
        }
    }

    println!("Collected {} points. Writing VDB...", points.len());
    ffi::write_vdb(output.to_str().context("Invalid output path")?, &points)?;
    println!("Done!");
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Some(path) = args.commonmcobj_source {
        let file = fs::File::open(&path)?;
        let reader = BufReader::new(file);
        let parsed_header = parse_header(reader);

        if let Some(header) = parsed_header {
            convert_world(
                header.export_bounds_min,
                header.export_bounds_max,
                PathBuf::from(header.world_path),
                path.with_extension("vdb"),
                &args.remap,
                args.solid,
                header.export_offset,
            )?;
        } else {
            anyhow::bail!("Passed file must have a CommonMCOBJ header!");
        }
    } else {
        if args.start.len() != 3 || args.end.len() != 3 {
            anyhow::bail!("Both --start and --end must have 3 coordinates: x,y,z");
        }

        let min_x = args.start[0].min(args.end[0]);
        let max_x = args.start[0].max(args.end[0]);
        let min_y = args.start[1].min(args.end[1]);
        let max_y = args.start[1].max(args.end[1]);
        let min_z = args.start[2].min(args.end[2]);
        let max_z = args.start[2].max(args.end[2]);

        if let (Some(w), Some(o)) = (args.world, args.output) {
            println!("Converting Minecraft world: {:?}", w);
            println!(
                "Bounds: ({}, {}, {}) to ({}, {}, {})",
                min_x, min_y, min_z, max_x, max_y, max_z
            );

            convert_world(
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
                w,
                o,
                &args.remap,
                args.solid,
                (args.offset[0], args.offset[1], args.offset[2]),
            )?;
        } else {
            anyhow::bail!("--world and --output are required if not using a CommonMCOBJ source");
        }
    }

    Ok(())
}

/// Returns true if the block is djacent to any transparency.
fn is_block_visible(world: &mut World, x: i32, y: i32, z: i32) -> Result<bool> {
    const NEIGHBORS: [(i32, i32, i32); 6] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ];

    for (dx, dy, dz) in NEIGHBORS {
        let n_block = world.get_block_name(x + dx, y + dy, z + dz)?;
        if is_transparent(&n_block) {
            return Ok(true);
        }
    }
    Ok(false)
}

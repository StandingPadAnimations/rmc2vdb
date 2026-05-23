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
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::thread;

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

    /// Number of threads to use for processing.
    #[arg(short, long, default_value_t = 1)]
    threads: usize,
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

        assert!(self.axes.len() == 3);
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
    world_path: PathBuf,
    output: PathBuf,
    remap: &str,
    solid: bool,
    offset: (f32, f32, f32),
    threads: usize,
) -> Result<()> {
    let coord_mapper = CoordinateMapper::new(remap)?;

    let (min_x, min_y, min_z) = min_bounds;
    let (max_x, max_y, max_z) = max_bounds;

    let total_voxels =
        (max_x - min_x + 1) as u64 * (max_y - min_y + 1) as u64 * (max_z - min_z + 1) as u64;

    let progress = AtomicU64::new(0);
    let last_percent = AtomicU32::new(0);

    let points = thread::scope(|s| {
        let mut handles = Vec::new();
        let x_range = max_x - min_x + 1;
        let threads = threads.max(1);
        let chunk_size = (x_range as f32 / threads as f32).ceil() as i32;

        for i in 0..threads {
            let start_x = min_x + i as i32 * chunk_size;
            let end_x = (start_x + chunk_size - 1).min(max_x);

            if start_x > max_x {
                break;
            }

            let world_path = &world_path;
            let coord_mapper = &coord_mapper;
            let progress = &progress;
            let last_percent = &last_percent;

            let handle = s.spawn(move || -> Result<Vec<ffi::VdbPoint>> {
                let mut world = World::new(world_path);
                let mut thread_points = Vec::new();

                let update_progress = || {
                    let current = progress.fetch_add(1, Ordering::Relaxed) + 1;
                    let p = (current * 100 / total_voxels.max(1)) as u32;
                    let l = last_percent.load(Ordering::Relaxed);
                    if p >= l + 5 {
                        let next = (p / 5) * 5;
                        if last_percent
                            .compare_exchange(l, next, Ordering::Relaxed, Ordering::Relaxed)
                            .is_ok()
                        {
                            println!("Progress: {}%", next);
                        }
                    }
                };

                for x in start_x..=end_x {
                    for y in min_y..=max_y {
                        for z in min_z..=max_z {
                            let (block, biome) = world.get_block_and_biome(x, y, z)?;
                            if is_transparent(&block) {
                                update_progress();
                                continue;
                            }

                            if !solid && !is_block_visible(&mut world, x, y, z)? {
                                update_progress();
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

                            thread_points.push(ffi::VdbPoint {
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
                            update_progress();
                        }
                    }
                }
                Ok(thread_points)
            });
            handles.push(handle);
        }

        let mut all_points = Vec::new();
        for handle in handles {
            let p = match handle.join() {
                Ok(Ok(p)) => p,
                Ok(Err(e)) => return Err(e),
                Err(_) => anyhow::bail!("Thread panicked"),
            };
            all_points.extend(p);
        }

        Ok(all_points)
    })?;

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
                args.threads,
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
                args.threads,
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

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

use anyhow::Result;
use fastanvil::biome::Biome;
use fastanvil::{Chunk, JavaChunk, Region};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Caches and provides an interface to Minecraft region and chunk data.
pub struct World {
    region_dir: PathBuf,
    regions: HashMap<(i32, i32), Option<Region<File>>>,
    chunks: HashMap<(i32, i32), Option<JavaChunk>>,
}

pub mod blocks {
    pub const AIR: &str = "minecraft:air";
    pub const CAVE_AIR: &str = "minecraft:cave_air";
    pub const VOID_AIR: &str = "minecraft:void_air";
    pub const BARRIER: &str = "minecraft:barrier";
    pub const PLAINS_BIOME: &str = "minecraft:plains";
}

impl World {
    /// Creates a new world interface for the given Minecraft world folder.
    pub fn new(world_dir: &Path) -> Self {
        Self {
            region_dir: world_dir.join("region"),
            regions: HashMap::new(),
            chunks: HashMap::new(),
        }
    }

    /// Loads or retrieves a cached Minecraft chunk from disk.
    pub fn get_chunk(&mut self, chunk_x: i32, chunk_z: i32) -> Result<Option<&JavaChunk>> {
        if self.chunks.contains_key(&(chunk_x, chunk_z)) {
            return Ok(self.chunks.get(&(chunk_x, chunk_z)).unwrap().as_ref());
        }

        let region_x = chunk_x.div_euclid(32);
        let region_z = chunk_z.div_euclid(32);

        let region_entry = self.regions.entry((region_x, region_z)).or_insert_with(|| {
            let path = self
                .region_dir
                .join(format!("r.{}.{}.mca", region_x, region_z));
            File::open(path)
                .ok()
                .and_then(|f| Region::from_stream(f).ok())
        });

        let chunk = region_entry.as_mut().and_then(|region| {
            let rel_chunk_x = chunk_x.rem_euclid(32) as usize;
            let rel_chunk_z = chunk_z.rem_euclid(32) as usize;
            region
                .read_chunk(rel_chunk_x, rel_chunk_z)
                .ok()
                .flatten()
                .and_then(|data| JavaChunk::from_bytes(&data).ok())
        });

        self.chunks.insert((chunk_x, chunk_z), chunk);
        Ok(self.chunks.get(&(chunk_x, chunk_z)).unwrap().as_ref())
    }

    /// Returns the namespaced block ID at the world-space coordinates.
    pub fn get_block_name(&mut self, x: i32, y: i32, z: i32) -> Result<String> {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z)? {
            let rel_x = x.rem_euclid(16) as usize;
            let rel_z = z.rem_euclid(16) as usize;
            if let Some(block) = chunk.block(rel_x, y as isize, rel_z) {
                return Ok(block.name().to_string());
            }
        }
        Ok(blocks::AIR.to_string())
    }

    /// Returns both the block name and biome ID at the world-space coordinates.
    pub fn get_block_and_biome(&mut self, x: i32, y: i32, z: i32) -> Result<(String, String)> {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);

        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z)? {
            let rel_x = x.rem_euclid(16) as usize;
            let rel_z = z.rem_euclid(16) as usize;

            let block_name = chunk
                .block(rel_x, y as isize, rel_z)
                .map(|b| b.name().to_string())
                .unwrap_or_else(|| blocks::AIR.to_string());

            let biome = chunk
                .biome(rel_x, y as isize, rel_z)
                .unwrap_or(Biome::Plains);

            // Convert PascalCase enum to snake_case namespaced ID
            let biome_name = format!("minecraft:{}", self.to_snake_case(&format!("{:?}", biome)));

            return Ok((block_name, biome_name));
        }
        Ok((blocks::AIR.to_string(), blocks::PLAINS_BIOME.to_string()))
    }

    fn to_snake_case(&self, s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.chars().enumerate() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('_');
                }
                result.push(c.to_lowercase().next().unwrap());
            } else {
                result.push(c);
            }
        }
        result
    }
}

/// Returns true if the block is air or an invisible barrier.
pub fn is_transparent(name: &str) -> bool {
    matches!(
        name,
        blocks::AIR | blocks::CAVE_AIR | blocks::VOID_AIR | blocks::BARRIER
    )
}

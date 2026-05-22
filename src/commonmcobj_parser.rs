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

// Rust port of the CommonMCOBJ parser I made for MCprep

use std::io::BufRead;
use std::str::FromStr;

pub const MAX_SUPPORTED_VERSION: i32 = 1;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CommonMCOBJTextureType {
    #[default]
    Atlas,
    IndividualTiles,
}

impl FromStr for CommonMCOBJTextureType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ATLAS" => Ok(CommonMCOBJTextureType::Atlas),
            "INDIVIDUAL_TILES" => Ok(CommonMCOBJTextureType::IndividualTiles),
            _ => Err(()),
        }
    }
}

/// Rust representation of the CommonMCOBJ header
#[derive(Debug, Clone)]
pub struct CommonMCOBJ {
    /// Version of the CommonMCOBJ spec
    pub version: i32,
    /// Exporter name in all lowercase
    pub exporter: String,
    /// Name of source world
    pub world_name: String,
    /// Path of source world*
    pub world_path: String,
    /// Min values of the selection bounding box
    pub export_bounds_min: (i32, i32, i32),
    /// Max values of the selection bounding box
    pub export_bounds_max: (i32, i32, i32),
    /// Offset from (0, 0, 0)
    pub export_offset: (f32, f32, f32),
    /// Scale of each block in meters; by default, this should be 1 meter
    pub block_scale: f64,
    /// Coordinate offset for blocks
    pub block_origin_offset: (f32, f32, f32),
    /// Is the Z axis of the OBJ considered up?
    pub z_up: bool,
    /// Are the textures using large texture atlases or individual textures?
    pub texture_type: CommonMCOBJTextureType,
    /// Are blocks split by type?
    pub has_split_blocks: bool,
    /// Original header
    pub original_header: Option<String>,
}

impl Default for CommonMCOBJ {
    fn default() -> Self {
        Self {
            version: 0,
            exporter: "NULL".to_string(),
            world_name: "NULL".to_string(),
            world_path: "NULL".to_string(),
            export_bounds_min: (0, 0, 0),
            export_bounds_max: (0, 0, 0),
            export_offset: (0.0, 0.0, 0.0),
            block_scale: 0.0,
            block_origin_offset: (0.0, 0.0, 0.0),
            z_up: false,
            texture_type: CommonMCOBJTextureType::Atlas,
            has_split_blocks: false,
            original_header: None,
        }
    }
}

fn clean_and_extract(line: &str) -> Option<(&str, &str)> {
    let (key_part, value_part) = line.split_once(':')?;
    // Find the position of the first alphabetic character
    let pos = key_part
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(0);
    Some((&key_part[pos..], value_part.trim()))
}

fn parse_tuple_i32(val: &str) -> Option<(i32, i32, i32)> {
    if val.len() >= 2 && val.starts_with('(') && val.ends_with(')') {
        let inner = &val[1..val.len() - 1];
        let mut parts = inner.split(", ");
        let a = parts.next()?.parse().ok()?;
        let b = parts.next()?.parse().ok()?;
        let c = parts.next()?.parse().ok()?;
        Some((a, b, c))
    } else {
        None
    }
}

fn parse_tuple_f32(val: &str) -> Option<(f32, f32, f32)> {
    if val.len() >= 2 && val.starts_with('(') && val.ends_with(')') {
        let inner = &val[1..val.len() - 1];
        let mut parts = inner.split(", ");
        let a = parts.next()?.parse().ok()?;
        let b = parts.next()?.parse().ok()?;
        let c = parts.next()?.parse().ok()?;
        Some((a, b, c))
    } else {
        None
    }
}

pub fn parse_common_header(header_lines: &[String]) -> CommonMCOBJ {
    let mut header = CommonMCOBJ::default();

    for line in header_lines {
        if !line.contains(':') {
            continue;
        }

        if let Some((key, value)) = clean_and_extract(line) {
            match key {
                "version" => {
                    if let Ok(v) = value.parse::<i32>() {
                        header.version = v;
                        if v > MAX_SUPPORTED_VERSION {
                            header.original_header = Some(header_lines.join("\n"));
                        }
                    }
                }
                "exporter" => header.exporter = value.to_string(),
                "world_name" => header.world_name = value.to_string(),
                "world_path" => header.world_path = value.to_string(),
                "export_bounds_min" => {
                    if let Some(t) = parse_tuple_i32(value) {
                        header.export_bounds_min = t;
                    }
                }
                "export_bounds_max" => {
                    if let Some(t) = parse_tuple_i32(value) {
                        header.export_bounds_max = t;
                    }
                }
                "export_offset" => {
                    if let Some(t) = parse_tuple_f32(value) {
                        header.export_offset = t;
                    }
                }
                "block_scale" => {
                    if let Ok(v) = value.parse::<f64>() {
                        header.block_scale = v;
                    }
                }
                "block_origin_offset" => {
                    if let Some(t) = parse_tuple_f32(value) {
                        header.block_origin_offset = t;
                    }
                }
                "z_up" => {
                    header.z_up = value == "true";
                }
                "texture_type" => {
                    if let Ok(t) = value.parse::<CommonMCOBJTextureType>() {
                        header.texture_type = t;
                    }
                }
                "has_split_blocks" => {
                    header.has_split_blocks = value == "true";
                }
                _ => {} // Ignore unknown keys
            }
        }
    }

    header
}

/// Parses a file and returns a CommonMCOBJ object if the header exists.
pub fn parse_header<R: BufRead>(reader: R) -> Option<CommonMCOBJ> {
    let mut header: Vec<String> = Vec::new();
    let mut found_header = false;
    let mut lines_read = 0;

    for line_result in reader.lines() {
        let raw_line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };

        // Similar to Python's " ".join(_l.rstrip().split())
        let tl: String = raw_line
            .trim_end()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        lines_read += 1;

        if lines_read > 100 && !tl.is_empty() && !tl.starts_with('#') {
            break; // No need to parse further than the true header area
        } else if tl == "# COMMON_MC_OBJ_START" {
            header.push(tl);
            found_header = true;
            continue;
        } else if tl == "# COMMON_MC_OBJ_END" {
            header.push(tl);
            break;
        }

        if !found_header || tl == "#" {
            continue;
        }

        header.push(tl);
    }

    if header.is_empty() {
        None
    } else {
        Some(parse_common_header(&header))
    }
}

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

use std::collections::HashMap;
use std::sync::OnceLock;

/// Information about a Minecraft biome's climate and visual properties.
#[derive(Debug, Clone, Copy)]
pub struct BiomeInfo {
    pub temperature: f32,
    pub downfall: f32,
    pub water_color: [f32; 3],
}

const DEFAULT_WATER_COLOR: (f32, f32, f32) = (0.247, 0.463, 0.894);
pub const DEFAULT_BIOME_INFO: BiomeInfo = BiomeInfo {
    temperature: 0.5,
    downfall: 0.5,
    water_color: [0.247, 0.463, 0.894],
};

static BIOME_MAP: OnceLock<HashMap<&'static str, BiomeInfo>> = OnceLock::new();

// Biome corners, based on Mineways
const GRASS_BIOME_CORNERS: [(u32, u32, u32); 3] = [(191, 183, 85), (128, 180, 151), (71, 205, 51)];
const FOLIAGE_BIOME_CORNERS: [(u32, u32, u32); 3] = [(174, 164, 42), (96, 161, 123), (26, 191, 0)];

pub fn get_biome_map() -> &'static HashMap<&'static str, BiomeInfo> {
    BIOME_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        let mut add = |name: &'static str, t: f32, d: f32, w: [f32; 3]| {
            m.insert(
                name,
                BiomeInfo {
                    temperature: t,
                    downfall: d,
                    water_color: w,
                },
            );
        };

        let std_water = [0.247, 0.463, 0.894];

        // Standard Overworld (Updated from Mineways biomes.cpp)
        add("minecraft:ocean", 0.5, 0.5, std_water);
        add("minecraft:plains", 0.8, 0.4, std_water);
        add("minecraft:desert", 2.0, 0.0, std_water);
        add("minecraft:windswept_hills", 0.2, 0.3, std_water);
        add("minecraft:forest", 0.7, 0.8, std_water);
        add("minecraft:taiga", 0.25, 0.8, std_water);
        add("minecraft:swamp", 0.8, 0.9, [0.380, 0.482, 0.392]);
        add("minecraft:river", 0.5, 0.5, std_water);
        add("minecraft:frozen_ocean", 0.0, 0.5, [0.247, 0.345, 0.761]);
        add("minecraft:frozen_river", 0.0, 0.5, std_water);
        add("minecraft:snowy_plains", 0.0, 0.5, std_water);
        add("minecraft:mushroom_fields", 0.9, 1.0, std_water);
        add("minecraft:beach", 0.8, 0.4, std_water);
        add("minecraft:jungle", 0.95, 0.9, std_water);
        add("minecraft:sparse_jungle", 0.95, 0.8, std_water);
        add("minecraft:deep_ocean", 0.5, 0.5, std_water);
        add("minecraft:stony_shore", 0.2, 0.3, std_water);
        add("minecraft:snowy_beach", 0.05, 0.3, std_water);
        add("minecraft:birch_forest", 0.6, 0.6, std_water);
        add("minecraft:dark_forest", 0.7, 0.8, std_water);
        add("minecraft:snowy_taiga", -0.5, 0.4, std_water);
        add("minecraft:old_growth_pine_taiga", 0.3, 0.8, std_water);
        add("minecraft:savanna", 1.2, 0.0, std_water); // Updated temp 1.2 from source
        add("minecraft:savanna_plateau", 1.0, 0.0, std_water);
        add("minecraft:badlands", 2.0, 0.0, std_water);
        add("minecraft:wooded_badlands", 2.0, 0.0, std_water);

        // 1.18+ and Variants
        add("minecraft:sunflower_plains", 0.8, 0.4, std_water);
        add("minecraft:ice_spikes", 0.0, 0.5, std_water);
        add("minecraft:old_growth_birch_forest", 0.6, 0.6, std_water);
        add("minecraft:old_growth_spruce_taiga", 0.25, 0.8, std_water);
        add("minecraft:windswept_savanna", 1.1, 0.0, std_water); // Source uses 1.1
        add("minecraft:eroded_badlands", 2.0, 0.0, std_water);
        add("minecraft:bamboo_jungle", 0.95, 0.9, std_water);
        add("minecraft:meadow", 0.5, 0.8, std_water);
        add("minecraft:grove", -0.2, 0.8, std_water);
        add("minecraft:snowy_slopes", -0.3, 0.9, std_water);
        add("minecraft:frozen_peaks", -0.7, 0.9, std_water);
        add("minecraft:jagged_peaks", -0.7, 0.9, std_water);
        add("minecraft:stony_peaks", 1.0, 0.3, std_water);
        add("minecraft:cherry_grove", 0.5, 0.8, std_water);
        add("minecraft:mangrove_swamp", 0.8, 0.9, [0.227, 0.408, 0.380]);
        add("minecraft:deep_dark", 0.8, 0.4, std_water);
        add("minecraft:lush_caves", 0.5, 0.5, std_water);
        add("minecraft:dripstone_caves", 0.8, 0.4, std_water);

        // Oceans
        add("minecraft:warm_ocean", 0.5, 0.5, [0.263, 0.835, 0.933]);
        add("minecraft:lukewarm_ocean", 0.5, 0.5, [0.271, 0.655, 0.949]);
        add(
            "minecraft:deep_lukewarm_ocean",
            0.5,
            0.5,
            [0.271, 0.655, 0.949],
        );
        add("minecraft:cold_ocean", 0.5, 0.5, [0.231, 0.341, 0.839]);
        add("minecraft:deep_cold_ocean", 0.5, 0.5, [0.231, 0.341, 0.839]);
        add(
            "minecraft:deep_frozen_ocean",
            0.0,
            0.5,
            [0.247, 0.345, 0.761],
        );

        // Nether
        add("minecraft:nether_wastes", 2.0, 0.0, std_water);
        add("minecraft:soul_sand_valley", 2.0, 0.0, std_water);
        add("minecraft:crimson_forest", 2.0, 0.0, std_water);
        add("minecraft:warped_forest", 2.0, 0.0, std_water);
        add("minecraft:basalt_deltas", 2.0, 0.0, std_water);

        // The End
        add("minecraft:the_end", 0.5, 0.5, std_water);
        add("minecraft:small_end_islands", 0.5, 0.5, std_water);
        add("minecraft:end_midlands", 0.5, 0.5, std_water);
        add("minecraft:end_highlands", 0.5, 0.5, std_water);
        add("minecraft:end_barrens", 0.5, 0.5, std_water);
        add("minecraft:the_void", 0.5, 0.5, std_water);

        m
    })
}

/// Resolve the tint color for a specific block in a specific biome.
pub fn get_tint(block: &str, elevation: f32, biome: &str) -> (f32, f32, f32) {
    if block == "minecraft:water" {
        return get_biome_map()
            .get(biome)
            .map(|i| (i.water_color[0], i.water_color[1], i.water_color[2]))
            .unwrap_or(DEFAULT_WATER_COLOR);
    }

    let info = get_biome_map().get(biome).unwrap_or(&DEFAULT_BIOME_INFO);

    if is_grass(block) {
        return approximate_tint(
            info.temperature,
            info.downfall,
            elevation,
            GRASS_BIOME_CORNERS,
        );
    }

    if is_foliage(block) {
        return approximate_tint(
            info.temperature,
            info.downfall,
            elevation,
            FOLIAGE_BIOME_CORNERS,
        );
    }

    (1.0, 1.0, 1.0)
}

fn is_grass(block: &str) -> bool {
    matches!(
        block,
        "minecraft:grass_block"
            | "minecraft:short_grass"
            | "minecraft:tall_grass"
            | "minecraft:fern"
            | "minecraft:large_fern"
            | "minecraft:bush"
            | "minecraft:potted_fern"
            | "minecraft:sugar_cane"
    )
}

fn is_foliage(block: &str) -> bool {
    matches!(
        block,
        "minecraft:oak_leaves"
            | "minecraft:birch_leaves"
            | "minecraft:spruce_leaves"
            | "minecraft:jungle_leaves"
            | "minecraft:acacia_leaves"
            | "minecraft:dark_oak_leaves"
            | "minecraft:mangrove_leaves"
            | "minecraft:vines"
    )
}

/// Tinting algorithm derived from Mineways
fn approximate_tint(
    temp: f32,
    downfall: f32,
    elevation: f32,
    corners: [(u32, u32, u32); 3],
) -> (f32, f32, f32) {
    let adjusted_temp = (temp - elevation * 0.00166667).clamp(0.0, 1.0);
    let rainfall = downfall.clamp(0.0, 1.0) * adjusted_temp;

    let lambda: [f32; 3] = [adjusted_temp - rainfall, 1.0 - adjusted_temp, rainfall];
    let (mut red, mut green, mut blue) = (0.0, 0.0, 0.0);
    for i in 0..3 {
        red += lambda[i] * corners[i].0 as f32;
        green += lambda[i] * corners[i].1 as f32;
        blue += lambda[i] * corners[i].2 as f32;
    }

    let r = red.clamp(0.0, 255.0) / 255.0;
    let g = green.clamp(0.0, 255.0) / 255.0;
    let b = blue.clamp(0.0, 255.0) / 255.0;

    (r, g, b)
}

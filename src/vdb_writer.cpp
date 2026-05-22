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

#include <cmath>
#include <map>
#include <string>
#include <string_view>
#include <vector>

#include <openvdb/openvdb.h>

#include "vdb_writer.hpp"
#include "rmc2vdb/src/main.rs.h"

namespace {

/**
 * Manages the conversion of string identifiers to integer indices for VDB
 * storage.
 */
class IndexMapper {
public:
  /**
   * @brief Resolves a unique integer ID for a string.
   * @param name The block or biome name.
   * @return The 0-based integer index.
   */
  [[nodiscard]] int getIndex(const std::string_view name) {
    const auto it = nameToIdx.find(std::string(name));
    if (it != nameToIdx.end()) {
      return it->second;
    }

    const int newIdx = static_cast<int>(names.size());
    nameToIdx.emplace(std::string(name), newIdx);
    names.emplace_back(std::string(name));
    return newIdx;
  }

  /**
   * @brief Injects the index-to-string mapping into the grid metadata.
   * @param grid The grid to receive the metadata.
   * @param prefix The prefix for the metadata keys (e.g., "block_name_").
   */
  void writeMetadata(openvdb::GridBase &grid,
                     const std::string_view prefix) const {
    for (size_t i = 0; i < names.size(); ++i) {
      grid.insertMeta(std::string(prefix) + std::to_string(i),
                      openvdb::StringMetadata(names[i]));
    }
  }

private:
  std::map<std::string, int, std::less<>> nameToIdx;
  std::vector<std::string> names;
};

} // namespace

/**
 * Serializes the collected Minecraft voxel data into an OpenVDB volume.
 *
 * @param filename The destination file path.
 * @param points The slice of point data passed from Rust.
 */
void write_vdb(const rust::Str filename,
               const rust::Slice<const VdbPoint> points) {
  openvdb::initialize();

  const auto densityGrid = openvdb::FloatGrid::create(0.0f);
  densityGrid->setName("density");
  densityGrid->setGridClass(openvdb::GRID_FOG_VOLUME);

  const auto colorGrid =
      openvdb::Vec3fGrid::create(openvdb::Vec3f(0.0f, 0.0f, 0.0f));
  colorGrid->setName("color");
  colorGrid->setGridClass(openvdb::GRID_FOG_VOLUME);

  const auto biomeGrid = openvdb::Int32Grid::create(-1);
  biomeGrid->setName("biome_index");

  const auto temperatureGrid = openvdb::FloatGrid::create(0.0f);
  temperatureGrid->setName("temperature");
  temperatureGrid->setGridClass(openvdb::GRID_FOG_VOLUME);

  const auto downfallGrid = openvdb::FloatGrid::create(0.0f);
  downfallGrid->setName("downfall");
  downfallGrid->setGridClass(openvdb::GRID_FOG_VOLUME);

  IndexMapper blockMapper;
  IndexMapper biomeMapper;

  auto densityAccessor = densityGrid->getAccessor();
  auto colorAccessor = colorGrid->getAccessor();
  auto biomeAccessor = biomeGrid->getAccessor();
  auto temperatureAccessor = temperatureGrid->getAccessor();
  auto downfallAccessor = downfallGrid->getAccessor();

  for (const auto &p : points) {
    const openvdb::Coord coord(static_cast<int>(std::round(p.x)),
                               static_cast<int>(std::round(p.y)),
                               static_cast<int>(std::round(p.z)));

    densityAccessor.setValue(coord, 1.0f);
    colorAccessor.setValue(coord, openvdb::Vec3f(p.r, p.g, p.b));
    biomeAccessor.setValue(coord, biomeMapper.getIndex(std::string_view(
                                      p.biome.data(), p.biome.size())));
    temperatureAccessor.setValue(coord, p.temperature);
    downfallAccessor.setValue(coord, p.downfall);
  }

  // Embed name dictionaries in the density grid's metadata
  blockMapper.writeMetadata(*densityGrid, "block_name_");
  biomeMapper.writeMetadata(*densityGrid, "biome_name_");

  const auto transform = openvdb::math::Transform::createLinearTransform(1.0);
  densityGrid->setTransform(transform);
  colorGrid->setTransform(transform);
  biomeGrid->setTransform(transform);
  temperatureGrid->setTransform(transform);
  downfallGrid->setTransform(transform);

  openvdb::GridCPtrVec grids;
  grids.push_back(densityGrid);
  grids.push_back(colorGrid);
  grids.push_back(biomeGrid);
  grids.push_back(temperatureGrid);
  grids.push_back(downfallGrid);

  openvdb::io::File(std::string(filename)).write(grids);
}

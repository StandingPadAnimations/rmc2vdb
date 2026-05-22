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

fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/vdb_writer.cpp")
        .std("c++17")
        .warnings(true)
        .compile("vdb-bridge");

    println!("cargo:rustc-link-lib=openvdb");
    println!("cargo:rustc-link-lib=tbb");
    println!("cargo:rustc-link-lib=Imath-3_2");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/vdb_writer.cpp");
    println!("cargo:rerun-if-changed=src/vdb_writer.h");
}

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

use std::process::Command;

fn main() {
    Command::new("glslc")
        .arg("src\\shader.comp")
        .arg("-o")
        .arg("shader.spv")
        .status()
        .expect("Failed to run glslc");

    Command::new("spirv-opt")
        .arg("shader.spv")
        .arg("-O")
        .arg("-o")
        .arg("shader.spv")
        .status()
        .expect("Failed to optimize shader");

    Command::new("spirv-remap")
        .arg("--do-everything")
        .arg("-i")
        .arg("shader.spv")
        .arg("-o")
        .arg("shader.spv")
        .status()
        .expect("Failed to remap shader");

    // Re-run build script if specific files change
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src\\shader.comp");
    println!("cargo:rerun-if-changed=shader.spv");
}
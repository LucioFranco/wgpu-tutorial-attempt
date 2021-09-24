use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use fs_extra::copy_items;
    use fs_extra::dir::CopyOptions;
    use std::env;

    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=assets/*");
    println!("cargo:rerun-if-changed=shaders/*");

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("assets/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    SpirvBuilder::new(
        concat!(env!("CARGO_MANIFEST_DIR"), "/shaders"),
        "spirv-unknown-vulkan1.1",
    )
    .print_metadata(MetadataPrintout::Full)
    .build()?;

    Ok(())
}

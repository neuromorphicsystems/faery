fn main() -> std::io::Result<()> {
    {
        let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        println!("cargo:rerun-if-changed={cargo_manifest_dir}/src/x264.h",);
        println!("cargo:rustc-link-search=native={cargo_manifest_dir}/x264/");
        println!("cargo:rustc-link-lib=static=x264");
        if !std::process::Command::new("/bin/bash")
            .args([
                "configure",
                "--disable-cli",
                "--enable-static",
                "--disable-interlaced",
                "--bit-depth=8",
                "--enable-lto",
                "--enable-strip",
                "--disable-avs",
                "--disable-swscale",
                "--disable-lavf",
                "--disable-ffms",
                "--disable-gpac",
                "--disable-lsmash",
            ])
            .current_dir("x264")
            .status()
            .expect("Failed to spawn '/bin/bash configure' for x264")
            .success()
        {
            panic!("Failed to configure x264");
        }
        if !std::process::Command::new("make")
            .args(["clean"])
            .current_dir("x264")
            .status()
            .expect("Failed to spawn 'make clean' for x264")
            .success()
        {
            panic!("Failed to make clean x264");
        }
        if !std::process::Command::new("make")
            .current_dir("x264")
            .status()
            .expect("Failed to spawn 'make' for x264")
            .success()
        {
            panic!("Failed to make x264");
        }
        let bindings = bindgen::Builder::default()
            .header("src/x264.h")
            .generate()
            .expect("Unable to generate bindings");
        let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("x264_bindings.rs"))
            .expect("Couldn't write bindings");
    }
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=assets/windows.manifest");
    println!("cargo:rerun-if-changed=assets/RadioCrash.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_manifest(include_str!("assets/windows.manifest"))
            .set_icon("assets/RadioCrash.ico")
            .set("ProductName", "Radio Crash Desktop")
            .set("FileDescription", "Radio Crash Internet Radio Receiver")
            .compile()
            .expect("Windows resources");
    }
}

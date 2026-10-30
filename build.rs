fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/RadioCrash.ico")
            .set("ProductName", "Radio Crash Desktop")
            .set("FileDescription", "Radio Crash Internet Radio Receiver")
            .compile()
            .expect("Windows resources");
    }
}

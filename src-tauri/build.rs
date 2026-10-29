fn main() {
    tauri_build::build();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-search=native={}", std::env::var("OUT_DIR").unwrap());
        // tauri-winres links this resource to ordinary binaries only. The isolated
        // example also needs its existing Common Controls v6 activation manifest.
        println!("cargo:rustc-link-arg-examples={}/resource.lib", std::env::var("OUT_DIR").unwrap());
    }
}

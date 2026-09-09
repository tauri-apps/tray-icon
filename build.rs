fn main() {
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_LIBAPPINDICATOR");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_KSNI");

    let gtk_platform = matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("linux" | "dragonfly" | "freebsd" | "netbsd" | "openbsd")
    );

    if gtk_platform
        && std::env::var_os("CARGO_FEATURE_LIBAPPINDICATOR").is_some()
        && std::env::var_os("CARGO_FEATURE_KSNI").is_some()
    {
        println!(
            "cargo::warning=features `libappindicator` and `ksni` are enabled together; tray-icon will use the `ksni` backend"
        );
    }
}

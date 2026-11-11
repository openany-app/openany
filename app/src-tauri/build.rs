fn main() {
    app_version();
    tauri_build::build()
}

/// Die App-Version aus `tauri.<plattform>.conf.json` als `OPENANY_APP_VERSION`
/// -- auch fuer die Hintergrunddienste, die ohne Tauri-App starten und die
/// Version sonst nicht kennten (B37: der Server weist zu alte Apps ab).
fn app_version() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let datei = format!("tauri.{os}.conf.json");
    println!("cargo:rerun-if-changed={datei}");
    let version = std::fs::read_to_string(&datei)
        .ok()
        .and_then(|text| {
            let nach = text.split("\"version\"").nth(1)?;
            let wert = nach.split('"').nth(1)?;
            Some(wert.to_string())
        })
        .unwrap_or_default();
    println!("cargo:rustc-env=OPENANY_APP_VERSION={version}");
}

// Auf Windows kein Konsolenfenster im Freigabebau.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    openany_app_lib::starten()
}

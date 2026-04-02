#![windows_subsystem = "windows"]

fn main() {
    #[cfg(not(debug_assertions))]
    {
        if !wiclive_lib::elevation::is_elevated() {
            wiclive_lib::elevation::relaunch_elevated();
            return;
        }
    }

    if std::env::args().any(|a| a == "--uninstall") {
        wiclive_lib::uninstall();
        return;
    }

    wiclive_lib::run();
}

#![windows_subsystem = "windows"]

fn main() {
    #[cfg(not(debug_assertions))]
    {
        if !wiclive_lib::elevation::is_elevated() {
            wiclive_lib::elevation::relaunch_elevated();
            return;
        }
    }

    wiclive_lib::run();
}

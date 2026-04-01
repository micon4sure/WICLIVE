#![windows_subsystem = "windows"]

fn main() {
    #[cfg(target_os = "windows")]
    {
        if !wiclive_lib::elevation::is_elevated() {
            wiclive_lib::elevation::relaunch_elevated();
            return;
        }
    }

    wiclive_lib::run();
}

use std::env;
fn main() {
    let build_env = env::var("BUILD_ENV").unwrap_or_else(|_| "development".to_string());
    println!("BUILD_ENV: {}", build_env);

    tauri_build::build()
}

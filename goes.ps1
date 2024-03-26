$Action = $args[0]
if ($args.Count -gt 1) {
    $Environment = $args[1]
} else {
    $Environment = "staging"
}


switch ($Action) {
    "run" {
        Write-Host "RUNNING in env $Environment"
        $env:WICLIVE_ENV="$Environment"; bun run tauri dev 
    }
    "build" {
        Write-Host "BUILDING in env $Environment"
        $env:WICLIVE_ENV="$Environment";
        $env:TAURI_PRIVATE_KEY=$(cat src-tauri/tauri-sign.key);
        $env:TAURI_KEY_PASSWORD="";
        bun run tauri build --ci
    }
}
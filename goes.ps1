$action = $args[0]
if ($args.Count -gt 1) {
    $environment = $args[1]
} else {
    $environment = "staging"
}


switch ($Action) {
    "run" {
        Write-Host "RUNNING in env $environment"
        $env:WICLIVE_ENV="$environment"; bun run tauri dev 
    }
    "build" {
        Write-Host "BUILDING in env $environment"
        $env:WICLIVE_ENV="$environment";
        $env:TAURI_PRIVATE_KEY=$(cat src-tauri/tauri-sign.key);
        $env:TAURI_KEY_PASSWORD="";
        bun run tauri build --ci
    }
    "act" {
        $key = Get-Content src-tauri/tauri-sign.key
        $token = Get-Content .github/token
        .\act.exe --action-offline-mode -P windows-latest=-self-hosted -j build-and-release-local -s TAURI_PRIVATE_KEY="$key" -s GITHUB_TOKEN="$token"
    }
}
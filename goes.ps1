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
    "runRelease" {
        Write-Host "RUNNING RELEASE in env $environment"
        $env:WICLIVE_ENV="$environment"; bun run tauri dev --release
    }
    "build" {
        Write-Host "BUILDING in env $environment"
        $env:WICLIVE_ENV="$environment";
        $env:TAURI_PRIVATE_KEY=$(cat src-tauri/tauri-sign.key);
        $env:TAURI_KEY_PASSWORD="";
        if($environment -eq "development" -or $environment -eq "staging") {
            # bun run tauri build --debug --ci
            bun run tauri build --debug -b none
        } else {
            bun run tauri build --ci
        }
    }
    "act" {
        $key = Get-Content src-tauri/tauri-sign.key
        $token = Get-Content .github/token
        .\act.exe --action-offline-mode -P windows-latest=-self-hosted -j build-and-release-local -s TAURI_PRIVATE_KEY="$key" -s GITHUB_TOKEN="$token"
    }
}
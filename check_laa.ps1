$path = "d:\2_GAMES\WIC\wic.exe"
if (-not (Test-Path $path)) { Write-Host "File not found: $path"; exit }

try {
    $stream = [System.IO.File]::OpenRead($path)
    $buffer = New-Object byte[] 4096
    $count = $stream.Read($buffer, 0, 4096)
    $stream.Close()

    if ($count -lt 64) { Write-Host "File too small"; exit }

    $pe_offset = [BitConverter]::ToInt32($buffer, 0x3C)
    
    if ($pe_offset + 24 -gt $count) { 
        Write-Host "PE header out of read range (Offset: $pe_offset)"
        exit 
    }

    $characteristics_offset = $pe_offset + 4 + 18
    $characteristics = [BitConverter]::ToUInt16($buffer, $characteristics_offset)
    $is_laa = ($characteristics -band 0x0020) -eq 0x0020
    
    Write-Host "Large Address Aware: $is_laa"
} catch {
    Write-Host "Error: $_"
}

param(
    [Parameter(Mandatory=$true)]
    [string]$ExePath
)

# Create a WScript.Shell COM object
$WScriptShell = New-Object -ComObject WScript.Shell

# Define the desktop path and the full shortcut path
$desktopPath = [System.IO.Path]::Combine($env:USERPROFILE, "Desktop")
$shortcutPath = [System.IO.Path]::Combine($desktopPath, "World in Conflict.lnk")

# Create the shortcut
$shortcut = $WScriptShell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $ExePath    # Use the provided executable path
$shortcut.Description = "World in Conflict"
$shortcut.Save()

Write-Output "Shortcut created at $shortcutPath"

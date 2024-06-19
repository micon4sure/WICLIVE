FROM mcr.microsoft.com/windows:ltsc2019

COPY vc_redist.x64.exe /
RUN vc_redist.x64.exe /quiet /norestart

RUN powershell -c "irm bun.sh/install.ps1|iex"

COPY vs_buildtools.exe /
RUN vs_buildtools.exe --quiet --wait --norestart \
    --installPath C:\BuildTools \
    --includeRecommended \
    --add Microsoft.VisualStudio.Workload.ManagedDesktopBuildTools || IF "%ERRORLEVEL%"=="3010" EXIT 0

RUN vs_buildtools.exe --quiet --wait --norestart \
    --installPath C:\BuildTools \
    --includeRecommended \
    --add Microsoft.VisualStudio.Workload.MSBuildTools || IF "%ERRORLEVEL%"=="3010" EXIT 0

RUN vs_buildtools.exe --quiet --wait --norestart \
    --installPath C:\BuildTools \
    --includeRecommended \
    --add Microsoft.VisualStudio.Workload.VCTools || IF "%ERRORLEVEL%"=="3010" EXIT 0

RUN vs_buildtools.exe --quiet --wait --norestart \
    --installPath C:\BuildTools \
    --includeRecommended \
    --add Microsoft.Component.VC.Runtime.UCRTSDK || IF "%ERRORLEVEL%"=="3010" EXIT 0

RUN C:\BuildTools\VC\Auxiliary\Build\vcvarsall.bat

RUN powershell -Command \
    Invoke-WebRequest -Uri https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile rustup-init.exe; \
    Start-Process .\rustup-init.exe -ArgumentList '-y' -NoNewWindow -Wait; \
    Remove-Item -Force rustup-init.exe; \
    [System.Environment]::SetEnvironmentVariable('Path', $env:Path + ';C:\Users\ContainerUser\.cargo\bin', [System.EnvironmentVariableTarget]::Machine)
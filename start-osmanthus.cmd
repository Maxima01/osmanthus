@echo off
setlocal
rem Works from Command Prompt, PowerShell, or a double-click in File Explorer.
pushd "%~dp0"
if errorlevel 1 exit /b 1
if exist "%~dp0target\debug\osmanthus-desktop.exe" (
    "%~dp0target\debug\osmanthus-desktop.exe"
) else (
    call "%~dp0scripts\cargo-local.cmd" run --locked -p osmanthus-desktop
)
set "OSMANTHUS_EXIT_CODE=%ERRORLEVEL%"
popd
if not "%OSMANTHUS_EXIT_CODE%"=="0" (
    echo.
    echo Osmanthus could not start. Review the error above.
    pause
)
exit /b %OSMANTHUS_EXIT_CODE%


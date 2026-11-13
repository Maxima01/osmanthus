@echo off
setlocal
rem Command Prompt entry point for the workspace-local Rust toolchain.
for %%I in ("%~dp0..") do set "OSMANTHUS_ROOT=%%~fI"
set "RUSTUP_HOME=%OSMANTHUS_ROOT%\tmp\toolchain\rustup"
set "CARGO_HOME=%OSMANTHUS_ROOT%\tmp\toolchain\cargo"
set "OSMANTHUS_NATIVE_BIN=%RUSTUP_HOME%\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained"
if not exist "%CARGO_HOME%\bin\cargo.exe" (
    echo Workspace-local Rust toolchain not found under tmp\toolchain.
    echo Restore the local toolchain, or use an installed Cargo with its matching C compiler.
    exit /b 1
)
set "OSMANTHUS_COMPILER_BIN="
for /d %%D in ("%OSMANTHUS_ROOT%\tmp\toolchain\llvm-mingw-*") do if not defined OSMANTHUS_COMPILER_BIN set "OSMANTHUS_COMPILER_BIN=%%~fD\bin"
if not defined OSMANTHUS_COMPILER_BIN (
    echo Workspace-local C compiler not found under tmp\toolchain.
    exit /b 1
)
set "CC=%OSMANTHUS_COMPILER_BIN%\clang.exe"
set "AR=%OSMANTHUS_COMPILER_BIN%\llvm-ar.exe"
set "PATH=%CARGO_HOME%\bin;%OSMANTHUS_NATIVE_BIN%;%OSMANTHUS_COMPILER_BIN%;%PATH%"
for %%I in ("%OSMANTHUS_NATIVE_BIN%\..\rust-lld.exe") do set "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=%%~fI"
set "RUSTFLAGS=-C linker-flavor=ld.lld -C link-self-contained=yes -C dlltool=llvm-dlltool.exe"
"%CARGO_HOME%\bin\cargo.exe" %*
exit /b %ERRORLEVEL%


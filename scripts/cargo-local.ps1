# Optional workspace-local toolchain. A normal installed Cargo also works.
$CargoArguments = $args
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$env:RUSTUP_HOME = Join-Path $projectRoot 'tmp\toolchain\rustup'
$env:CARGO_HOME = Join-Path $projectRoot 'tmp\toolchain\cargo'
$cargoBin = Join-Path $env:CARGO_HOME 'bin'
$rustNativeBin = Join-Path $env:RUSTUP_HOME 'toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin\self-contained'
$portableCompiler = Get-ChildItem (Join-Path $projectRoot 'tmp\toolchain') -Directory -Filter 'llvm-mingw-*' -ErrorAction SilentlyContinue | Select-Object -First 1
if ($portableCompiler) {
    $compilerBin = Join-Path $portableCompiler.FullName 'bin'
    $env:CC = Join-Path $compilerBin 'clang.exe'
    $env:AR = Join-Path $compilerBin 'llvm-ar.exe'
    $env:PATH = "$cargoBin;$rustNativeBin;$compilerBin;$env:PATH"
} else {
    $env:PATH = "$cargoBin;$rustNativeBin;$env:PATH"
}
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = Join-Path (Split-Path $rustNativeBin -Parent) 'rust-lld.exe'
$env:RUSTFLAGS = '-C linker-flavor=ld.lld -C link-self-contained=yes -C dlltool=llvm-dlltool.exe'
& (Join-Path $cargoBin 'cargo.exe') @CargoArguments
exit $LASTEXITCODE

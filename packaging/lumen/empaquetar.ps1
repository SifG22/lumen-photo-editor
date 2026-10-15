# Empaqueta Lumen para Windows en un ZIP listo para descargar: dist\Lumen-<version>-windows-<arch>.zip
#
#   powershell -ExecutionPolicy Bypass -File packaging\lumen\empaquetar.ps1                # x64, compila si hace falta
#   powershell -ExecutionPolicy Bypass -File packaging\lumen\empaquetar.ps1 -Arch arm64    # ARM64 (necesita las herramientas ARM64 de MSVC)
#   ... -NoBuild    # empaqueta el .exe ya compilado sin llamar a cargo
#
# El ZIP lleva Lumen.exe, las licencias (obligatorias: Lumen está basado en PhotoCraft, MIT) y un LEEME.

param(
    [ValidateSet("x64", "arm64")] [string]$Arch = "x64",
    [switch]$NoBuild,
    # Tambien el instalador (Lumen-Setup-<version>-<arch>.exe), con Inno Setup 6.
    [switch]$Instalador,
    # ISCC.exe de Inno Setup, si no esta en su sitio habitual.
    [string]$Iscc = "",
    # Donde se publicaran los instaladores: con -Instalador se escribe dist\latest.json (el aviso de
    # actualizacion que lee Lumen, apps/photocraft/src/updater.rs) apuntando ahi.
    [string]$BaseUrl = "https://sifg.net/lumen",
    [string]$Notas = ""
)
$ErrorActionPreference = "Stop"
$root = Resolve-Path "$PSScriptRoot\..\.."
$version = (Select-String -Path "$root\crates\ui-egui\src\brand.rs" -Pattern 'pub const VERSION: &str = "([^"]+)"').Matches[0].Groups[1].Value
$triple = if ($Arch -eq "arm64") { "aarch64-pc-windows-msvc" } else { "x86_64-pc-windows-msvc" }

if (-not $NoBuild) {
    Push-Location $root
    try {
        if ($Arch -eq "arm64") { cargo build --release -p photocraft --target $triple } else { cargo build --release -p photocraft }
        if ($LASTEXITCODE -ne 0) { throw "La compilacion ha fallado" }
    } finally { Pop-Location }
}

# x64 nativo sale en target\release; el resto en target\<triple>\release.
$exe = @("$root\target\release\photocraft.exe", "$root\target\$triple\release\photocraft.exe") |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if ($Arch -eq "arm64") { $exe = "$root\target\$triple\release\photocraft.exe" }
if (-not (Test-Path $exe)) { throw "No encuentro $exe; compila primero" }

# DXC, el compilador de shaders moderno de Microsoft: junto a Lumen.exe la app lo usa en vez de FXC
# (gpu_startup::dx12_compiler) y la primera imagen se abre unas 2,5 veces mas rapido. Se descarga una
# vez de la release oficial, fijada por version y huella SHA-256.
$dxcUrl = "https://github.com/microsoft/DirectXShaderCompiler/releases/download/v1.9.2609/dxc_2026_09_29.zip"
$dxcSha = "ad31b1fc8443175d204f77a611fdb3ef2ec42759bdc2f1167368de24a4a7e7f1"
$cache = "$root\dist\cache"
New-Item -ItemType Directory -Force $cache | Out-Null
$dxcZip = "$cache\dxc_2026_09_29.zip"
if (-not (Test-Path $dxcZip) -or (Get-FileHash $dxcZip -Algorithm SHA256).Hash.ToLower() -ne $dxcSha) {
    Invoke-WebRequest -Uri $dxcUrl -OutFile $dxcZip -UserAgent "Photocraft-dev"
}
if ((Get-FileHash $dxcZip -Algorithm SHA256).Hash.ToLower() -ne $dxcSha) { throw "La descarga de DXC no coincide con su SHA-256" }
$dxcDir = "$cache\dxc_2026_09_29"
if (-not (Test-Path "$dxcDir\bin")) { Expand-Archive -Path $dxcZip -DestinationPath $dxcDir -Force }

$name = "Lumen-$version-windows-$Arch"
$stage = "$root\dist\$name"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$stage\licencias" | Out-Null
Copy-Item $exe "$stage\Lumen.exe"
Copy-Item "$dxcDir\bin\$Arch\dxcompiler.dll", "$dxcDir\bin\$Arch\dxil.dll" "$stage\"
New-Item -ItemType Directory -Force "$stage\licencias\DirectXShaderCompiler" | Out-Null
Copy-Item "$dxcDir\LICENCE-MIT.txt", "$dxcDir\LICENSE-LLVM.txt", "$dxcDir\LICENSE-MS.txt" "$stage\licencias\DirectXShaderCompiler\"
Copy-Item "$root\LICENSE-MIT", "$root\LICENSE-APACHE", "$root\NOTICE", "$root\ATTRIBUTION.md" "$stage\licencias\"
Copy-Item "$root\assets\legal\TERMINOS.txt" "$stage\TERMINOS.txt"
@"
Lumen Photo Editor $version para Windows ($Arch), por SifG22 - https://sifg.net

Para empezar: abre Lumen.exe. No hace falta instalar nada.
Si Windows muestra "Windows protegio su PC", pulsa "Mas informacion" y "Ejecutar de todas formas"
(sale porque el programa aun no esta firmado digitalmente).

Al usar Lumen aceptas los terminos y condiciones de SifG22 (TERMINOS.txt).
Lumen esta basado en PhotoCraft (https://github.com/storytold/photocraft), usado bajo la licencia MIT.
Incluye DirectX Shader Compiler de Microsoft (dxcompiler.dll, dxil.dll), con sus licencias.
Las licencias estan en la carpeta "licencias".
"@ | Out-File -Encoding utf8 "$stage\LEEME.txt"

$zip = "$root\dist\$name.zip"
Remove-Item -Force $zip -ErrorAction SilentlyContinue
Compress-Archive -Path "$stage\*" -DestinationPath $zip
$hash = (Get-FileHash $zip -Algorithm SHA256).Hash.ToLower()
"$hash  $name.zip" | Out-File -Encoding ascii "$zip.sha256"
Write-Host "Listo: $zip"
Write-Host "SHA-256: $hash"

if ($Instalador) {
    $candidates = @($Iscc, "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe", "$env:ProgramFiles\Inno Setup 6\ISCC.exe", "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe") |
        Where-Object { $_ -and (Test-Path $_) }
    $compiler = $candidates | Select-Object -First 1
    if (-not $compiler) { throw "No encuentro ISCC.exe (Inno Setup 6); pasa -Iscc <ruta>" }
    & $compiler "/DVersion=$version" "/DArch=$Arch" "/DSource=$stage" "/DOutDir=$root\dist" "$PSScriptRoot\lumen.iss"
    if ($LASTEXITCODE -ne 0) { throw "Inno Setup ha fallado" }
    $setup = "$root\dist\Lumen-Setup-$version-$Arch.exe"
    $setupHash = (Get-FileHash $setup -Algorithm SHA256).Hash.ToLower()
    "$setupHash  Lumen-Setup-$version-$Arch.exe" | Out-File -Encoding ascii "$setup.sha256"
    Write-Host "Instalador: $setup"
    Write-Host "SHA-256: $setupHash"

    # latest.json: una entrada por arquitectura; la otra se conserva si es de la misma version.
    $manifestPath = "$root\dist\latest.json"
    $manifest = [ordered]@{ version = $version; notes = $Notas }
    if (Test-Path $manifestPath) {
        $old = Get-Content $manifestPath -Raw | ConvertFrom-Json
        if ($old.version -eq $version) {
            if (-not $Notas -and $old.notes) { $manifest.notes = $old.notes }
            foreach ($a in @("x64", "arm64")) { if ($a -ne $Arch -and $old.$a) { $manifest[$a] = $old.$a } }
        }
    }
    $manifest[$Arch] = [ordered]@{ url = "$BaseUrl/Lumen-Setup-$version-$Arch.exe"; sha256 = $setupHash }
    # Sin BOM: es JSON para otro programa.
    [System.IO.File]::WriteAllText($manifestPath, ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
    Write-Host "Aviso de actualizacion: $manifestPath (subelo a $BaseUrl/latest.json)"
}

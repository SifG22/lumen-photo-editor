@echo off
rem Compila Lumen Photo Editor y genera el instalador para la web:
rem   dist\Lumen-Setup-<version>-x64.exe  (el que se descarga la gente)
rem   dist\Lumen-<version>-windows-x64.zip (version portable)
rem   dist\latest.json                     (aviso de actualizacion: subelo a https://sifg.net/lumen/)
rem Usa pocos procesos a la vez (-j 2) para no quedarse sin memoria.
cd /d "%~dp0"
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
echo Compilando (puede tardar 20-30 minutos)...
cargo build --release -p photocraft -j 2
if errorlevel 1 (
  echo.
  echo La compilacion ha fallado. Revisa los errores de arriba.
  pause
  exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0packaging\lumen\empaquetar.ps1" -NoBuild -Instalador
if errorlevel 1 (
  echo.
  echo El empaquetado ha fallado. Revisa los errores de arriba.
  pause
  exit /b 1
)
echo.
echo Listo. Los archivos para tu web estan en la carpeta dist.
explorer "%~dp0dist"
pause

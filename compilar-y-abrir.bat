@echo off
rem Compila PhotoCraft (solo lo que haya cambiado) y lo a
rem Usa las Build Tools 2022: la instalacion "Insiders" de Visual Studio no trae msvcrt.lib.
cd /d "%~dp0"
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
rem Ajustes privados de los estilos (secrets\lumen-looks.txt, fuera de git): si estan, entran en la compilacion.
if exist "secrets\lumen-looks.txt" for /f "usebackq eol=# delims=" %%L in ("secrets\lumen-looks.txt") do set "LUMEN_LOOK_TROPICAL_SUNSET=%%L"
cargo build --release -p photocraft
if errorlevel 1 (
  echo.
  echo La compilacion ha fallado. Revisa los errores de arriba.
  pause
  exit /b 1
)
start "" "%~dp0target\release\photocraft.exe"

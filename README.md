<p align="center">
  <img alt="Lumen Photo Editor" src="assets/app-icon/hicolor/128x128/apps/ai.storyteller.photocraft.png" width="96">
</p>

<h1 align="center">Lumen Photo Editor</h1>

<p align="center">
  Editor de fotos para Windows, rápido y nativo, por <a href="https://sifg.net">SifG22</a>.
</p>

<p align="center">
  <a href="https://github.com/SifG22/lumen-photo-editor/releases/latest"><b>Descargar para Windows</b></a>
  · <a href="https://sifg.net">sifg.net</a>
  · <a href="assets/legal/TERMINOS.txt">Términos</a>
</p>

---

## Qué es

Lumen Photo Editor es un editor de fotos de escritorio para Windows, con capas, máscaras, ajustes,
filtros, pinceles, texto y formas, pensado para ser rápido incluso con imágenes grandes.

- **Capas de verdad**: capas de ajuste editables, máscaras, modos de fusión y estilos de capa.
- **Proyectos `.lumen`**: guardan todo (capas, máscaras, efectos) para seguir editando o compartir.
- **Abre y guarda PSD**, además de JPEG, PNG, WebP, TIFF, HEIC, RAW de cámara y más.
- **Redes sociales**: documentos y recortes listos para Instagram (1:1 y 4:5), Historias, Reels y
  TikTok (9:16), YouTube, X y Pinterest.
- **Estilos rápidos**: looks de un clic (Cálido, Frío, Vintage, B/N intenso, Cine, Pop) hechos con
  capas de ajuste que puedes retocar.
- **Tema oscuro propio**, botones de deshacer y rehacer a mano y actualizaciones automáticas.

## Descargar e instalar

1. Descarga el instalador de la [última versión](https://github.com/SifG22/lumen-photo-editor/releases/latest):
   - `Lumen-Setup-<versión>-x64.exe` para la mayoría de PCs (Intel o AMD).
   - `Lumen-Setup-<versión>-arm64.exe` para PCs con Windows en ARM (Snapdragon, Surface Pro X/11).
2. Ábrelo y pulsa **Instalar**. Lumen aparece en el menú Inicio; después puedes borrar el archivo descargado.
3. Si Windows muestra «Windows protegió su PC», pulsa «Más información» › «Ejecutar de todas formas»
   (la aplicación aún no está firmada digitalmente).

Requisitos: Windows 10 u 11 de 64 bits (x64 o ARM64). Lumen te avisa cuando hay una versión nueva
y se actualiza solo. Para desinstalarlo: Configuración › Aplicaciones › Lumen Photo Editor.

También hay una versión portable (`.zip`) que no necesita instalación.

## Compilar desde el código

Necesitas [Rust](https://rustup.rs) y las Build Tools de Visual Studio 2022 (C++).

```bat
generar-instalador.bat
```

Compila en modo release y deja en `dist\` el instalador, el ZIP portable y el `latest.json` del
aviso de actualización. Para solo compilar y abrir la aplicación: `compilar-y-abrir.bat`.

Las versiones publicadas las compila GitHub Actions (`.github/workflows/lumen-release.yml`) para
x64 y ARM64 al subir una etiqueta `vX.Y.Z`.

## Términos y licencias

- El uso de Lumen Photo Editor se rige por los [términos y condiciones de SifG22](assets/legal/TERMINOS.txt).
- El nombre, el icono y la marca Lumen son de SifG22.
- Lumen Photo Editor está basado en [PhotoCraft](https://github.com/storytold/photocraft), de los
  autores de PhotoCraft, usado bajo la licencia MIT ([LICENSE-MIT](LICENSE-MIT), [NOTICE](NOTICE)).
  Los componentes de terceros y sus licencias están en [ATTRIBUTION.md](ATTRIBUTION.md).
- Lumen no está afiliado ni respaldado por Adobe ni por los autores de PhotoCraft.

---

<p align="center">Hecho por <a href="https://sifg.net">SifG22</a></p>

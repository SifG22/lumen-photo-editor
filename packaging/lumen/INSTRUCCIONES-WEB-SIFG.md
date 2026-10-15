# Instrucciones para la IA que edita sifg.net: publicar Lumen Photo Editor

Copia este documento entero en el chat de la IA que mantiene la web **sifg.net**. Explica qué hay
que publicar para que la gente descargue **Lumen Photo Editor** (editor de fotos para Windows, de
SifG22) y para que las copias instaladas se **actualicen solas**.

## 1. Qué es y qué archivos hay

Lumen Photo Editor es una aplicación de escritorio para Windows (no una web). Cada versión produce
estos archivos (los genera `generar-instalador.bat` en la carpeta `dist` del proyecto de la app):

| Archivo | Para qué | ¿Va en la web? |
|---|---|---|
| `Lumen-Setup-<versión>-x64.exe` | Instalador para la mayoría de PCs (Intel/AMD) | **Sí, descarga principal** |
| `Lumen-Setup-<versión>-arm64.exe` | Instalador para PCs con Windows en ARM (Snapdragon, Surface Pro X/11) | Sí, si existe |
| `Lumen-<versión>-windows-x64.zip` | Versión portable (sin instalar) | Opcional |
| `latest.json` | Aviso de actualización que leen las apps instaladas | **Sí, ruta fija (ver §3)** |
| `*.sha256` | Huellas para comprobar la descarga | Opcional |

Versión actual: **0.0.1**.

## 2. Dónde alojar los instaladores (importante: tamaño)

El instalador pesa **~31 MB**. Si sifg.net se publica en **Cloudflare Pages**, no admite archivos de
más de **25 MiB**: el `.exe` no se puede poner dentro del sitio. Opciones:

- **A. GitHub Releases** (recomendado, gratis, sin límite de descargas): crear una Release `v0.0.1`
  en el repositorio de Lumen y adjuntar los instaladores. URL de descarga:
  `https://github.com/<usuario>/<repo>/releases/download/v0.0.1/Lumen-Setup-0.0.1-x64.exe`
- **B. Cloudflare R2** (o cualquier almacenamiento de archivos) con un dominio tipo
  `https://descargas.sifg.net/lumen/`.
- **C. El propio hosting de sifg.net**, solo si admite archivos de 31 MB o más.

Las URLs de los instaladores quedan grabadas en `latest.json` al generarlo; el desarrollador lo
genera con `empaquetar.ps1 -Instalador -BaseUrl "<carpeta donde estarán los instaladores>"`. Si los
instaladores acaban en otra carpeta, hay que regenerar `latest.json` (no editarlo a mano, salvo el
texto de `notes`).

## 3. El aviso de actualización (`latest.json`) — ruta FIJA

Las copias instaladas de Lumen consultan, como mucho una vez al día:

```
https://sifg.net/lumen/latest.json
```

Esa ruta está grabada en la aplicación: **no se puede cambiar** sin sacar una versión nueva. Requisitos:

- Publicar el `latest.json` de `dist` exactamente en `https://sifg.net/lumen/latest.json` (en un sitio
  Astro/estático: `public/lumen/latest.json`).
- Debe servirse por **HTTPS**, como JSON (`Content-Type: application/json` recomendado) y **sin
  redirecciones a HTML** ni páginas de error con código 200. Si no hay versión nueva, basta con dejar
  el de la versión actual.
- No cachearlo mucho tiempo (por ejemplo `Cache-Control: max-age=300`), para que las versiones nuevas
  se detecten pronto. En Cloudflare Pages se puede poner en `public/_headers`:
  ```
  /lumen/latest.json
    Cache-Control: public, max-age=300
    Content-Type: application/json; charset=utf-8
  ```
- Formato (lo genera el script; las URLs deben ser `https://` y el `sha256` el del archivo exacto):
  ```json
  {
    "version": "0.0.1",
    "notes": "Primera versión pública.",
    "x64":   { "url": "https://…/Lumen-Setup-0.0.1-x64.exe",   "sha256": "…64 caracteres hex…" },
    "arm64": { "url": "https://…/Lumen-Setup-0.0.1-arm64.exe", "sha256": "…" }
  }
  ```
  Si la `sha256` no coincide con el archivo descargado, la app **no instala** la actualización.

## 4. Página de descarga (crear en sifg.net)

Ruta sugerida: `https://sifg.net/lumen` (o `/lumen-photo-editor`). Contenido mínimo, en español:

- **Título**: «Lumen Photo Editor — editor de fotos para Windows».
- **Botón principal**: «Descargar para Windows (x64)» → instalador x64.
- **Enlace secundario**: «¿PC con procesador ARM (Snapdragon, Surface Pro X)? Descarga la versión
  ARM64» → instalador arm64. Y «Versión portable (ZIP)» si se publica.
- **Versión y tamaño**: «Versión 0.0.1 · 31 MB · Windows 10 y 11 de 64 bits».
- **Cómo se instala**: «Abre el archivo descargado y pulsa Instalar. Lumen aparecerá en el menú
  Inicio. Después puedes borrar el archivo descargado. Para desinstalar: Configuración ›
  Aplicaciones › Lumen Photo Editor.»
- **Aviso de Windows**: «Si aparece "Windows protegió su PC", pulsa "Más información" y "Ejecutar de
  todas formas". Sale porque la aplicación todavía no está firmada digitalmente.»
- **Actualizaciones**: «Lumen te avisa cuando hay una versión nueva y se actualiza solo.»
- **Novedades destacadas** (opcional): proyectos `.lumen` con todas las capas; formatos para redes
  sociales (Instagram 1:1 y 4:5, Historias/Reels/TikTok 9:16, YouTube); Estilos rápidos de un clic;
  tema oscuro propio; abre y guarda PSD.
- **Enlaces legales** (ver §5): «Términos y condiciones» · «Licencias».
- **Crédito** (pie de página): «Lumen Photo Editor está basado en PhotoCraft
  (https://github.com/storytold/photocraft), licencia MIT.» No decir que es de Adobe ni compatible
  oficialmente con Photoshop.

## 5. Páginas legales

- **Términos y condiciones**: publicar el contenido de `assets/legal/TERMINOS.txt` (del proyecto de
  la app) en `https://sifg.net/lumen/terminos`. La app los muestra en Ayuda › Acerca de › Términos y
  en el instalador; el texto de la web debe ser el mismo. Si se cambian, cambiar también ese archivo
  en la app.
- **Privacidad** (puede ir dentro de los términos): Lumen no envía imágenes ni datos personales; solo
  consulta `latest.json` una vez al día (la petición incluye la IP, como cualquier visita web).
- **Licencias**: enlazar a la licencia MIT de PhotoCraft y mencionar que el instalador incluye las
  licencias de todos los componentes en la carpeta `licencias`.

## 6. Cada vez que salga una versión nueva

1. El desarrollador sube la versión en la app (`crates/ui-egui/src/brand.rs`, `VERSION`) y ejecuta
   `generar-instalador.bat` (con `-BaseUrl` apuntando a donde irán los instaladores).
2. Subir los instaladores nuevos al alojamiento de §2.
3. **Después** de que los instaladores estén accesibles, sustituir `https://sifg.net/lumen/latest.json`
   por el nuevo (si se publica antes, las apps intentarán descargar un archivo que aún no existe).
4. Actualizar en la página de descarga el número de versión, el tamaño y los enlaces.

## 7. No hacer

- No cambiar la ruta `/lumen/latest.json` ni renombrar los campos del JSON.
- No editar a mano `url` ni `sha256` en `latest.json`.
- No subir el `.exe` a Cloudflare Pages (límite de 25 MiB).
- No usar el nombre o logotipo de PhotoCraft/ArtCraft como marca de Lumen (solo el crédito del pie).

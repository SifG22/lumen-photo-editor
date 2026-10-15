# Instrucciones para la IA que edita sifg.net: página de Lumen Photo Editor

Copia este documento entero en el chat de la IA que mantiene la web **sifg.net**.

## Resumen

**Lumen Photo Editor** es un editor de fotos de escritorio para Windows, de SifG22. Las descargas y
las actualizaciones las sirve **GitHub** (repositorio `SifG22/lumen-photo-editor`): sifg.net **no
aloja archivos** ni interviene en las actualizaciones. En sifg.net solo hay que crear una **página
de presentación y descarga** con enlaces a GitHub, y una página de **términos**.

## 1. Enlaces de descarga (fijos, siempre la última versión)

Estos enlaces no cambian nunca: GitHub los redirige a la versión más reciente.

| Para | Enlace |
|---|---|
| Instalador para la mayoría de PCs (Intel/AMD, **x64**) | `https://github.com/SifG22/lumen-photo-editor/releases/latest/download/Lumen-Setup-x64.exe` |
| Instalador para PCs con Windows en ARM (**ARM64**: Snapdragon, Surface Pro X/11) | `https://github.com/SifG22/lumen-photo-editor/releases/latest/download/Lumen-Setup-arm64.exe` |
| Todas las versiones, ZIP portables y notas | `https://github.com/SifG22/lumen-photo-editor/releases` |
| Código y README | `https://github.com/SifG22/lumen-photo-editor` |

No descargues los instaladores para subirlos a sifg.net: enlaza siempre a GitHub (el instalador pesa
unos 30 MB y Cloudflare Pages no admite archivos de más de 25 MiB).

## 2. Página de descarga (crear en sifg.net)

Ruta sugerida: `https://sifg.net/lumen` (o `/lumen-photo-editor`). Contenido, en español:

- **Título**: «Lumen Photo Editor — editor de fotos para Windows».
- **Subtítulo**: «Rápido, nativo y sin suscripciones. Capas, máscaras, ajustes, filtros y PSD.»
- **Botón principal**: «Descargar para Windows» → instalador x64 (§1).
- **Enlace secundario**: «¿PC con procesador ARM (Snapdragon, Surface Pro X)? Descarga la versión
  ARM64» → instalador ARM64. Y «Otras versiones y versión portable» → página de Releases.
- **Requisitos**: «Windows 10 u 11 de 64 bits · ~30 MB».
- **Cómo se instala**: «Abre el archivo descargado y pulsa Instalar. Lumen aparecerá en el menú
  Inicio; después puedes borrar el archivo descargado. Para desinstalar: Configuración ›
  Aplicaciones › Lumen Photo Editor.»
- **Aviso de Windows**: «Si aparece "Windows protegió su PC", pulsa "Más información" y "Ejecutar de
  todas formas". Sale porque la aplicación todavía no está firmada digitalmente.»
- **Actualizaciones**: «Lumen te avisa cuando hay una versión nueva y se actualiza solo.»
- **Funciones destacadas**: edición por capas y capas de ajuste editables; proyectos `.lumen`;
  abre y guarda PSD, JPEG, PNG, WebP, HEIC y RAW; formatos para redes sociales (Instagram 1:1 y 4:5,
  Historias/Reels/TikTok 9:16, YouTube); Estilos rápidos de un clic (Cálido, Frío, Vintage, B/N,
  Cine, Pop); tema oscuro propio; funciona sin conexión y no envía tus fotos a ningún sitio.
- **Captura**: se puede usar
  `https://raw.githubusercontent.com/SifG22/lumen-photo-editor/main/docs/lumen/captura.png`
  (o copiar esa imagen a la web, que pesa menos de 1 MB).
- **Icono**: `https://raw.githubusercontent.com/SifG22/lumen-photo-editor/main/assets/app-icon/hicolor/256x256/apps/ai.storyteller.photocraft.png`
- **Enlaces legales**: «Términos y condiciones» (§3) · «Licencias»
  (`https://github.com/SifG22/lumen-photo-editor/blob/main/ATTRIBUTION.md`).
- **Crédito** (pie de página, texto plano): «Lumen Photo Editor está basado en PhotoCraft
  (https://github.com/storytold/photocraft), licencia MIT.»
  No usar logos de PhotoCraft ni de ArtCraft; no decir que es de Adobe ni compatible oficialmente
  con Photoshop.

## 3. Términos y condiciones

Publica en `https://sifg.net/lumen/terminos` el texto de
`https://github.com/SifG22/lumen-photo-editor/blob/main/assets/legal/TERMINOS.txt` (es el mismo que
muestra la app y el instalador). Si se cambian, hay que cambiarlos también en la app.

## 4. Cuando salga una versión nueva

No hay que hacer nada en sifg.net: los enlaces de §1 apuntan solos a la última versión y las apps
instaladas se actualizan desde GitHub. Opcional: actualizar en la página el número de versión o las
novedades si se muestran.

# Proyecto Cubos

Una escena estilo "Minecraft" en Rust: generador de diorama compuesto por cubos (capas), renderer por trazado de rayos simplificado y visualización con raylib.

Este README resume cómo compilar y ejecutar el proyecto, las dependencias, controles de cámara, y notas sobre la escena (materiales y capas) para que sea fácil continuar desarrollándolo o integrarlo en un flujo de trabajo.

## Requisitos
- Rust (estable) con cargo instalado.
- Windows, Linux o macOS (probado en Windows en este workspace).
- Librerías nativas requeridas por `raylib` (en Windows suele estar empaquetado por la crate; en Linux/macOS instala raylib si es necesario).

## Dependencias (desde `Cargo.toml`)
- nalgebra-glm = "0.20.0"  — cálculo vectorial y transformaciones.
- rand = "0.9.2"           — utilidades aleatorias (si se usan en experimentos).
- raylib = "5.5.1"         — ventana y render 2D/3D para mostrar el framebuffer.
- rayon = "1.8"            — paralelismo en el render (rayos por pixel).

## Cómo compilar
Desde PowerShell (o tu shell preferido), en la raíz del proyecto:

```powershell
cargo build --release
```

Para ejecutar en modo release:

```powershell
cargo run --release
```

## Controles (teclado)
- Flechas Izquierda/Derecha/Arriba/Abajo: movimiento lateral/adelante/atrás.
- W / S: subir/bajar (elevación).
- A / D: girar la cámara (yaw) izquierda/derecha.
- Q / E: ajustar pitch (rotación arriba/abajo).
- Z / X o PageUp / PageDown: zoom (mover cámara en el eje Z relativo).
- R: alternar auto-rotación.
- Esc o cerrar ventana: salir.

## Estructura y módulos principales
- `src/main.rs` — punto de entrada. Construye la escena (funciones `build_reference_diorama` y `build_reference_diorama_layers`), gestiona la cámara, el bucle principal y el shading (funciones `cast_ray`, `cast_ray_rec`, `sample_sky`, `sample_material`).
- `src/framebuffer.rs` — estructura de framebuffer: mantiene un `Image`/`Texture` y la lógica de presentar pixels a la ventana (usa raylib internamente).
- `src/ray_intersect.rs` — tipos y trait para intersección rayo-objeto (`Material`, `Intersect`, `RayIntersect`).
- `src/cube.rs` — definición del `Cube` y lógica de intersección con AABB/rayo.
- `src/bvh.rs` — builder e intersección BVH (estructura `BVH`, `build_bvh`, `intersect_bvh`).
- `src/materials.rs` — fábricas de materiales (`material_grass`, `material_water`, `material_glass`, etc.).

## Materiales del diorama
El diorama está compuesto por cubos con diferentes materiales definidos en `src/materials.rs`. Cada material contiene propiedades como color difuso (`diffuse`), reflectividad y transparencia. Algunos materiales notables:
- Grass (verde) — difuso (80,180,70)
- Water (azul claro) — difuso (64,160,255) — usado para celdas de agua y gotas pequeñas
- Glass — material translúcido para ventanas
- Pillar/stone/darkwood — distintos tonos para pilares y madera oscura
- Pumpkin — material decorativo

La escena se genera por capas (capas representadas en el README):
- Capa 1 - Suelo (y=0.5)
- Capa 2 - Terrazas (y=1.5)
- Capa 3 - Casa (y entre 2.5 y 3.5)
- Capa 4 - Detalles (ornamentos, pilares)
- Capa 5 - Grilla / detalles altos (y=4.5)

Las capas se construyen de forma determinista (sin leer assets) y hay una opción para renderizar solo una capa usando `--layer N` en los argumentos (por ejemplo `cargo run --release -- --layer=3`).

## Notas sobre texturas y capturas
- Originalmente existía un gestor de texturas y captura a disco; esa funcionalidad fue eliminada intencionalmente y sustituida por muestreo simple (`sample_material` y `sample_sky`) para simplificar.

## Rendimiento
- El render de rayos usa `rayon` para paralelizar el cálculo por pixel. El rendimiento dependerá de la resolución y del `render_scale` aplicado en el render (por defecto se reduce el tamaño de render interno para acelerar).
- Prueba con `cargo run --release` y la ventana mostrará la escena en tiempo real; baja la resolución si necesitas más frames por segundo.


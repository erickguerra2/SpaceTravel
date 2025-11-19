# SpaceTravel — Software Renderer en Rust

Este proyecto es una simulación de un sistema planetario creada para el Proyecto final del curso de Gráficas por Computadora.
El render es completamente por software en Rust. Raylib solo se usa para mostrar el framebuffer, manejar input y cargar recursos.

Incluye:

* Sistema planetario con órbitas y rotación
* Cámara 3D con movimiento libre
* Warp instantáneo
* Nave propia siguiendo a la cámara
* Skybox cúbico
* Colisiones simples
* Render de mallas .obj

---

## Video del proyecto

[https://youtu.be/Q3tDOHBYDII?si=d5TXSKwWYs8lBfHz](https://youtu.be/Q3tDOHBYDII?si=d5TXSKwWYs8lBfHz)

---

## Cómo ejecutarlo

git clone <tu-repo>
cd SpaceTravel
cargo run

---

## Estructura del proyecto

SpaceTravel/
│── src/
│   ├── main.rs
│   ├── renderer.rs
│   ├── camera.rs
│   ├── planet.rs
│   ├── object.rs
│   ├── skybox.rs
│   ├── warp.rs
│   ├── movement.rs
│   ├── texture.rs
│   ├── utils.rs
│   └── math.rs
│
│── assets/
│   ├── models/
│   │   ├── sphere.obj
│   │   └── ship.obj
│   └── skybox/
│       ├── back.png
│       ├── front.png
│       ├── left.png
│       ├── right.png
│       ├── top.png
│       └── bottom.png
│
└── README.md

---

## Controles

WASD — mover la cámara
Mouse — rotación
1–5 — warp entre planetas


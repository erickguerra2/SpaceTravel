# SpaceTravel – Proyecto Final de Gráficas

Este proyecto es una simulación de un sistema planetario renderizado completamente con un **software renderer en CPU**, sin usar la GPU para dibujar modelos.  
Incluye un sol, varios planetas con órbitas y rotación, una nave que sigue a la cámara, un sistema de warp y un skybox cúbico.

## Video de demostración 🎥
https://youtu.be/Q3tDOHBYDII?si=d5TXSKwWYs8lBfHz

## Controles
- **W / A / S / D** → mover la cámara  
- **Mouse** → rotar cámara  
- **1–5** → Warp instantáneo 
- La nave sigue automáticamente la orientación de la cámara  
- Colisiones básicas para no atravesar planetas

## Estructura del proyecto
SpaceTravel/  
├── src/  
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
├── assets/  
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
└── README.md

## Características implementadas
- Renderizador de triángulos hecho a mano (CPU)
- Skybox cúbico cargado desde imágenes
- Sombras básicas y coloración de cuerpos celestes
- Orbitales circulares animadas
- Warp instantáneo con animación
- Nave 3D que sigue a la cámara
- Movimiento libre en 3D
- Colisión básica para evitar entrar a planetas

## Notas
Este proyecto fue desarrollado en **Rust + Raylib**, usando un mesh `.obj` para las esferas y la nave, pero dibujando cada triángulo manualmente en CPU.

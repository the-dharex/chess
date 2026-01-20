# Chess Game en Rust 🦀♟️

Un juego de Ajedrez completo escrito en Rust utilizando la librería gráfica `ggez`.

## Características

- **Jugar contra la IA**: Desafía a un oponente controlado por el ordenador (algoritmo Minimax con poda Alpha-Beta).
- **Multijugador Online (LAN/P2P)**: Juega contra un amigo en tu red local o a través de internet (si tienes puertos abiertos).
- **Interfaz Gráfica**:
    - Tablero y piezas renderizados con corrección de aspecto.
    - Indicadores de turno y movimientos válidos.
    - Rotación del tablero para el jugador con piezas negras.
- **Reglas Completas**:
    - Movimiento estándar de piezas.
    - Enroque (Castling).
    - Captura al paso (En Passant).
    - Promoción de peones (a Reina).
    - Detección de Jaque y Jaque Mate.
- **Utilidades**:
    - Generación automática de código de conexión (IP:Puerto).
    - Copiado automático al portapapeles al hospedar una partida.

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (cargo) instalado.

## Cómo ejecutar

1. Clona el repositorio.
2. Abre una terminal en la carpeta del proyecto.
3. Ejecuta el juego:

```bash
cargo run --release
```

> **Nota**: La versión `--release` está altamente recomendada para que la IA "piense" rápido.

## Cómo jugar Online

1. **Host (Anfitrión)**:
    - Selecciona **"2. Host Game"** en el menú.
    - El juego mostrará un código (tu IP y puerto). Este se copia automáticamente a tu portapapeles.
    - Comparte este código con tu amigo.
    - Espera a que se conecte.

2. **Join (Unirse)**:
    - Selecciona **"3. Join Game"**.
    - Escribe el código (IP:Puerto) que te dio el anfitrión.
    - Presiona **Enter**.

3. **Juego**:
    - Los colores (Blancas/Negras) se asignan aleatoriamente al comenzar la conexión.

## Estructura del Proyecto

- `src/main.rs`: Punto de entrada y configuración de la ventana.
- `src/game.rs`: Bucle principal, manejo de estados (Menú, Juego) y eventos.
- `src/board.rs`: Lógica del tablero, generación de movimientos y reglas.
- `src/pieces.rs`: Definición de piezas y colores.
- `src/ai.rs`: Inteligencia Artificial (Minimax).
- `src/network.rs`: Módulo de red para la comunicación TCP.
- `src/resources.rs`: Gestión de assets (imágenes y sonidos).

## Créditos

Desarrollado por thedharex en exclusivo para el mundo.

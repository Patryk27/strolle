# strolle

Strolle (coming from _strålspårning_) is an experimental real-time renderer that
supports global illumination:

<p align="center">
  <img width="512" height="512" src="_readme/demo-v3.jpg" />
</p>

Our goal is to create an engine that is able to produce a good-looking image on
a consumer GPU without having to rely on hardware ray-tracing capabilities.

Strolle comes integrated with [Bevy](https://bevyengine.org/), but can be also
used on its own (through `wgpu`).

Status: work in progress, no official release yet; examples below should work on
Windows, Mac & Linux (with WebGPU support possible in the future).

## Examples & Demo

``` shell
$ cargo build-shaders
$ cargo run --release --example demo
```

Shows a little dungeon-like tech demo, as in the render above.

Use WASD to move, mouse to navigate the camera; extra controls include:

- 1: Switch camera to the default mode,
- 2: Switch camera to a direct-lightning-only mode,
- 3: Switch camera to a indirect-lightning-only mode,
- 4: Switch camera to a normals-only mode,
- 5: Switch camera to a bvh-heatmap mode,
- 0: Switch camera to use Bevy's renderer,
- Semicolon: Toggle camera's controls on/off; useful for taking screenshots,
- O/P: Adjust sun's altitude.

Model thanks to:    
https://sketchfab.com/3d-models/low-poly-game-level-82b7a937ae504cfa9f277d9bf6874ad2

## License

MIT License

Copyright (c) 2022-2023 Patryk Wychowaniec & Jakub Trąd

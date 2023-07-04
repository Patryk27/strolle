# strolle

Strolle (coming from _strålspårning_) is an experimental pixel-art~ish real-time
renderer that supports global illumination:

<p align="center">
  <img width="512" height="512" src="_readme/demo-v4.jpg" />
</p>

Our goal is to create an engine that is able to produce a good-looking image on
a consumer GPU without having to rely on hardware ray-tracing capabilities.

Strolle comes integrated with [Bevy](https://bevyengine.org/), but can be also
used on its own (through `wgpu`).

Status: work in progress, no official release yet; examples below should work on
Windows, Mac & Linux (with WebGPU support possible in the future).

## Examples & Demo

``` shell
$ cargo run --release --example demo
```

Shows a dungeon tech demo, as in the render above.

Use WASD to move and mouse to navigate the camera; extra controls include:

- 1: Switch camera to the default mode,
- 2: Switch camera to a direct lightning only mode,
- 3: Switch camera to a demodulated direct lightning only mode,
- 4: Switch camera to an indirect lightning only mode,
- 5: Switch camera to a demodulated indirect lightning only mode,
- 6: Switch camera to a normals-only mode,
- 7: Switch camera to a velocity-only mode,
- 8: Switch camera to a bvh-heatmap mode,
- 0: Switch camera to Bevy's renderer,
- F: Toggle flashlight,
- Semicolon: Toggle camera's controls on/off - useful for taking screenshots,
- O/P: Adjust sun's altitude.

Model thanks to:    
https://sketchfab.com/3d-models/low-poly-game-level-82b7a937ae504cfa9f277d9bf6874ad2

## License

MIT License

Copyright (c) 2022-2023 Patryk Wychowaniec & Jakub Trąd

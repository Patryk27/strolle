# strolle

Strolle (coming from _strålspårning_) is an experimental real-time renderer that
supports global illumination:

![cornell.png](_readme/cornell.png)

Our goal is to create an engine that is able to produce a good-looking image on
a consumer GPU without having to rely on hardware ray-tracing capabilities.

Strolle comes integrated with [Bevy](https://bevyengine.org/), but can be also
used on its own (through `wgpu`).

Status: work in progress, no official release yet; examples below should work on
Windows, Mac & Linux (with WebGPU support possible in the future).

## Examples

Before running any example, run (just once):

``` shell
$ cargo build-shaders
```

### Cornell

``` shell
$ cargo run --release --example cornell
```

Shows the (in)famous Cornell Box:

![cornell.png](_readme/cornell.png)

### Dungeon

``` shell
$ cargo run --release --example dungeon
```

Shows a little dungeon tech demo, with textures, normal mapping and whatnot:

![dungeon.png](_readme/dungeon.png)

Use WASD to move, mouse to navigate the camera; extra controls include:

- 1: Switch camera to default mode (i.e. from the options below),
- 2: Switch camera to direct-lightning-only mode,
- 3: Switch camera to indirect-lightning-only mode,
- 4: Switch camera to normal-only mode,
- 0: Switch camera to use Bevy's renderer,
- Semicolon: Toggle camera's controls on/off; useful for taking screenshots.

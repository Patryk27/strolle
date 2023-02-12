# strolle

Strolle (coming from _strålspårning_) is an experimental real-time renderer that
supports global illumination:

![cornell.png](_readme/cornell.png)

Our goal is to create a path-tracer that doesn't rely on hardware raytracing
capabilities and is able to generate an approximate, good-looking image on a 
relatively modern, consumer GPU.

Strolle comes integrated with [Bevy](https://bevyengine.org/), but can be also
used on its own (through `wgpu`).

Status: work in progress, no official release yet; examples below should work on
Windows, Mac & Linux (with WebGPU support possible in the future).

## Examples

Before running any example, please execute (just once):

``` shell
$ cargo build-shaders
```

### Cameras

``` shell
$ cargo run --release --example cameras
```

Shows support for multiple cameras; the four cameras there show the rasterized
view, the raytraced view, normals, and BVH nodes.

### Cornell

``` shell
$ cargo run --release --example cornell
```

Shows the famous Cornell Box.

### Cubes

``` shell
$ cargo run --release --example cubes
```

Shows a few cubes rotating around the origin; use keyboard & mouse to move the
camera.

### Glass

``` shell
$ cargo run --release --example glass
```

Shows support for refraction; use keyboard & mouse to move the camera.

### Instancing

``` shell
$ cargo run --release --example instancing
```

Shows support for hierarchical BVH - there's 100 bunnies * 69k polygons per each
bunny; use keyboard & mouse to move the camera.

### Models

``` shell
$ cargo run --release --example models
```

Shows support for complex geometry (from a few thousand to a few hundred
thousand triangles); use left and right arrows to change models, use keyboard &
mouse to move the camera.

Models thanks to:
- https://github.com/alecjacobson/common-3d-test-models
- https://github.com/RayMarch/ferris3d

### Nefertiti

``` shell
$ cargo run --release --example nefertiti
```

Renders a 2 million-polygon head of Nefertiti with some dynamic lightning; use
keyboard & mouse to move the camera.

(note that model takes a few seconds to appear.)

Model thanks to: https://www.cs.cmu.edu/~kmcrane/Projects/ModelRepository/.

### Textures

``` shell
$ cargo run --release --example textures
```

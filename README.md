# vapoursynth-webp

WebP encoding as a [VapourSynth][] plugin. Currently does VP8L lossless encoding.

## Install

Via [vsrepo][]:

```
vsrepo install webp
```

Or manually: download a release from the [Releases][] page and unzip
`webp.dll` (Windows), `libwebp.so` (Linux), or `libwebp.dylib` (macOS) into a
[plugins directory][plugin-autoloading]. There are separate artifacts for Raptor
Lake (`*-raptorlake.zip`) and AMD Zen 4 (`*-znver4.zip`) CPUs which may or may
not have better performance than the plain x86_64 build.

## API

```python
webp.WebP(
    clip: vs.VideoNode,
    path: str,
    parents: bool = False,
)
```

- `clip` — Input video node. Must have RGB24 format.
- `path` — Output file path. Supports [Rust-like formatting][]. The frame number
  `n` is the only available parameter.
- `parents` — If `True`, any missing parents of `path` are created as needed.
  Otherwise, a missing parent will throw an error.

Writes each requested frame to `path`.

## Examples

Write frame 5000 to `output.webp`:

```python
from vskernels import Bilinear
from vssource import BestSource
from vstools import core, vs

clip = BestSource.source(r"/path/to/video.mkv")
clip = Bilinear.resample(clip, format=vs.RGB24)

core.webp.WebP(clip, path="output.webp").get_frame(5000)
```

Write 10 frames to `output_N.webp`, where `N` is the frame number
(`output_5000.webp`, `output_5001.webp`, etc.):

```python
from vskernels import Bilinear
from vssource import BestSource
from vstools import clip_async_render, core, vs

clip = BestSource.source(r"/path/to/video.mkv")
clip = Bilinear.resample(clip, format=vs.RGB24)

webp = core.webp.WebP(clip, path=r"output_{n}.webp")
clip_async_render(webp[5000:5010])
```

Because Rust-like formatting is supported, things like padding work as well:

```python
# Pad to 5 digits with 0s (`output_05000.webp`, `output_05001.webp`, etc.)
core.webp.WebP(clip, path=r"output_{n:05}.webp")

# Include the frame number twice (`output_5000_5000.webp`, `output_5001_5001.webp`, etc.)
core.webp.WebP(clip, path=r"output_{n}_{n}.webp")
```

Like Python's pathlib, by default an error will be thrown if any of `path`'s
parents do not exist. This filter can optionally create any missing parents with
the `parents` parameter:

```python
# This will throw an error.
core.webp.WebP(clip, path=r"/nonexistent/path/output.webp")

# But this will create the missing parents.
core.webp.WebP(clip, path=r"/nonexistent/path/output.webp", parents=True)

# This also enables patterns like creating a folder per frame.
core.webp.WebP(clip, path=r"screenshots/frame_{n}/output.webp", parents=True)
```

## Build

Rust v1.83.0-nightly and cargo may be used to build the project. Older versions
will likely work fine but they aren't explicitly supported.

```bash
$ git clone https://github.com/sgt0/vapoursynth-webp.git
$ cd vapoursynth-webp

# Debug build.
$ cargo build

# Release (optimized) build.
$ cargo build --release

# Release build optimized for the host CPU.
$ RUSTFLAGS="-C target-cpu=native" cargo build --release
```

[VapourSynth]: https://www.vapoursynth.com
[vsrepo]: https://github.com/vapoursynth/vsrepo
[Releases]: https://github.com/sgt0/vapoursynth-webp/releases
[plugin-autoloading]: https://www.vapoursynth.com/doc/installation.html#plugin-autoloading
[Rust-like formatting]: https://doc.rust-lang.org/std/fmt/index.html#formatting-parameters

Development moved to https://github.com/aestream/faery.

## Contribute

Local build (first run).

```sh
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install maturin numpy
maturin develop  # or maturin develop --release to build with optimizations
```

Local build (subsequent runs).

```sh
cd python
source .venv/bin/activate
maturin develop  # or maturin develop --release to build with optimizations
```

After changing any of the files in _framebuffers_.

```sh
flatc --rust -o src/aedat/ flatbuffers/*.fbs
```

Before pushing new code, run the following to lint and format it.

```sh
cd python
isort .; black .; pyright .
```

Files needed to complete testing:

-   longer recordings?
-   EVT2.1 (GenX320)
-   Dat 1 (2D, CD, Trigger)
-   Dat 2 (2D, Trigger)

Formats not implemented yet:

-   .dvs (13 bytes packed)
-   .dvs.br (13 bytes packed, brotli-encoded)
-   AEDAT 1
-   AEDAT 2
-   AEDAT 3

Test big endian platform

TODO

-   Implement functions to read the time range from files without generating events (this should speed dup any function that needs the time range)

```sh
flatc --rust -o src/aedat/ flatbuffers/*.fbs
```

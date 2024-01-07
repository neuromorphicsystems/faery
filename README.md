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

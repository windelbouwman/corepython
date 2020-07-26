
# CorePython

A python to webassembly compiler written in rust.

Features:

- Embeddable in browser (small WebAssembly download).
- Very minimal python language. Only the core of it, nothing fancy, like micropython.
- Precompile python into wasm offline, and ship this wasm file.

# Example

```python

def myAdd(a: int, b: int) -> int:
    return a + b

```

TODO: how to include in javascript?

```
<script type="bin/wasm" src="corepython.wasm">
<script type="text/python">

def myAdd(a: int, b: int) -> int:
    return a + b

</script>
```

# Usage

Compile demo.py into a wasm file locally:

    $ cargo run
    $ ls demo.wasm
    $ python run_it.py

Or view the wasm in the browser:

    $ python -m http.server
    $ open test_page.html in the browser

Or use the wasm build of corepython, and use it from npm:

    $ cd corepython-wasm
    $ wasm-pack build
    $ cd www
    $ npm install
    $ npm run start

# Design topics

Below is a list of python concepts and how they map to WebAssembly.

- How to implement python `int`? For now, it is mapped to WebAssembly `i32`.
- How to implement python `str`? TODO, check javascript bindgen
- How to implement python `float`? Map it to `f64`.
- How to implement python `list`? TODO
- How to implement python trickery like `eval`, `exec`, `sys.setprofile`? We don't.

# Planning

- Simple types, `int`, `float` and functions.
- `str` support

# Motivation

This is a prototype to answer this question: https://snarky.ca/what-is-the-core-of-the-python-programming-language/


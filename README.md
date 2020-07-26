
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

    $ cargo run
    $ ls demo.wasm
    $ python run_it.py

Or view the wasm in the browser:

    $ python -m http.server
    $ open test_page.html in the browser

# Planning

- Simple types, `int`, `float` and functions.
- `str` support

# Motivation

This is a prototype to answer this question: https://snarky.ca/what-is-the-core-of-the-python-programming-language/


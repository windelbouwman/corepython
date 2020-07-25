
# CorePython

A python to webassembly compiler written in rust.

Features:

- Embeddable in webbrowser (small WebAssembly download).
- Very minimal python language. Only the core of it, nothing fancy, like micropython.
- Procompile python into wasm offline, and ship this wasm file.

# Example

```python

def myAdd(a: int, b: float) -> float:
    return a + b

```

TODO: how to include in javascript?

```
<script type="bin/wasm" src="corepython.wasm">
<script type="text/python">

def myAdd(a: int, b: float) -> float:
    return a + b

</script>
```

# Usage

    $ cargo install corepython
    $ corepython compile demo.py
    $ ls demo.wasm


# Planning

- Simple types, `int`, `float` and functions.
- `str` support

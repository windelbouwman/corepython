
# CorePython

A [Python](https://www.python.org/) to [WebAssembly](https://webassembly.org/)
compiler written in [Rust](https://www.rust-lang.org/).

Features:

- Very minimal subset of the Python language. Only the core of it, nothing fancy.
- CorePython compiler itself is embeddable in browser (small WebAssembly download).

# Usage

First clone this repository, and check the Python to WebAssembly compiler options:

    $ git clone <this repo>
    $ cargo run -- -h  # All options after -- are passed to the CorePython compiler
    ...

Say, you start with an annotated function like this:

```python

def myAdd(a: int, b: int) -> int:
    return a + b

```

Put this function in `demo.py`, and compile `demo.py` into a WebAssembly file:

    $ cargo run -- demo.py
    ...  # No time for coffee at this point :(
    $ ls demo.wasm

Now you can run this WebAssembly file as usual, using one of the many runtimes,
such as [nodejs](https://nodejs.org/en/about/) or [ppci](https://ppci.readthedocs.io/en/latest/):

    $ node run_demo.js
    $ python run_demo.py

Or view the wasm in the browser:

    $ python -m http.server
    $ open test_page.html in the browser

Since the CorePython compiler is written in rust, you can use the WebAssembly build of
CorePython, and use it client side:

    $ cd corepython-wasm
    $ wasm-pack build
    $ cd www
    $ npm install
    $ npm run start

# Design topics

Below is a list of python concepts and how they map to WebAssembly.
This will also contain idea's behind implementation of various
language constructs.

## How is Python's `int` implemented?

For now, it is mapped to WebAssembly `i32`. Other options are `i64`
or support infinite size integers (how?).

## How is Python's `str` implemented?

Open topic.
TODO, check javascript bindgen for rust?

## How is Python's `float` implemented?

It is mapped to `f64`.

## How is Python's `list` implemented?

This is an open topic. Initial idea is to go for Python's new
syntax:

```python
def myFunc(x: list[int]):
    pass
```

Representation of the list in memory is to be determined.

TODO

## How are Python's magic functions like `eval`, `exec`, `sys.setprofile` implemented?

They are not implemented. In order to stick to the essence of what the Python
language is, those function are not available. If it cannot be compiled to
WebAssembly, it cannot be supported.

## How is Python's `class` supported?

This is an open topic. Classes will be analyzed at compile time, and there
fields and methods will be determined. Then they will be layed out in memory
in some way.

# Planning

- Simple types, `int`, `float` and functions.
- `str` support

# Motivation

This is a prototype to answer this question: https://snarky.ca/what-is-the-core-of-the-python-programming-language/


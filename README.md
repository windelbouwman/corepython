
# CorePython

A [Python](https://www.python.org/) to [WebAssembly](https://webassembly.org/)
compiler written in [Rust](https://www.rust-lang.org/).

Features:

- Very minimal subset of the Python language. Only the core of it, nothing fancy.
- CorePython compiler itself is embeddable in browser (small WebAssembly download).

[![Build Status](https://github.com/windelbouwman/corepython/workflows/build/badge.svg)](https://github.com/windelbouwman/corepython/actions)

# Phase

This project is in prototype phase.

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

# Performance

Well, okay, that's nice, but how fast is it?

Good question! I did some completely non-scientific early tests with
the mandelbrot example (`mandel.py`). Compiled it to WebAssembly and
ran it with both node and ppci.

- Python version: 17 ms.
- Python -> WebAssembly -> native (using ppci): 2.7 ms.
- Python -> WebAssembly -> node: 20 ms.

You can try this using the `run_mandel.js` and `run_mandel.py` scripts:

    $ cargo run -- mandel.py  # Compile mandel.py
    $ python run_mandel.py
    $ node run_mandel.js

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

This is an open topic. Initial idea is to go for Python's new (3.9+)
syntax:

```python
def myFunc(x: list[int]):
    y = [1, 2, 3]  # Type of y will be list[int]
```

Representation of the list in memory is a single `i32` with the length
of the list, followed by the elements of the list. A list object
is passed around as a single `i32` value which points to the memory
where the list is residing.

This might change in the future, when support for list extending is added.
Open issues:
- How to append items? Reallocate the memory?

## How are Python's magic functions like `eval`, `exec`, `sys.setprofile` implemented?

They are not implemented. In order to stick to the essence of what the Python
language is, those function are not available. If it cannot be compiled to
WebAssembly, it cannot be supported.

## How is Python's `class` supported?

This is an open topic. Classes will be analyzed at compile time, and there
fields and methods will be determined. Then they will be layed out in memory
in some way.

# Planning

- [x] Python `int` support.
- [x] Python `float` support.
- [x] function `def` support.
- [ ] Python `str` support.
- [ ] Python `list` support.
- [ ] Python `class` support.
- [ ] The rest (TM).

# Motivation

This is a prototype to answer this question: https://snarky.ca/what-is-the-core-of-the-python-programming-language/


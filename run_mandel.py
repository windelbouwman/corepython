
from ppci.wasm import instantiate, read_wasm

import mandel
import x
import time

with open('mandel.wasm', 'rb') as f:
    module = read_wasm(f)

inst = instantiate(module, {'x': {'putc': x.putc}})

print(inst)
print('python -> wasm -> native code mandel:')
t1 = time.time()
inst.exports['mandel']()
t2 = time.time()

print('Python mandel:')
t3 = time.time()
mandel.mandel()
t4 = time.time()

dt_native = t2 - t1
dt_python = t4 - t3
print('native took:', dt_native, 'python took:', dt_python, 'speedup', dt_python / dt_native)


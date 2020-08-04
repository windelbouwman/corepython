
from ppci.wasm import instantiate, read_wasm

import mandel
import x
import time

with open('mandel2.wasm', 'rb') as f:
    module = read_wasm(f)

inst = instantiate(
    module,
    {
    'x': {
        'putc': x.putc,
        'put_float': x.put_float
    }
})

print(inst)
print('mandel():')
inst.exports['mandel']()
print()

print('mandel2():')
inst.exports['mandel2']()
print()

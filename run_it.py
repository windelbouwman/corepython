
from ppci.wasm import instantiate, read_wasm

with open('demo.wasm', 'rb') as f:
    module = read_wasm(f)

inst = instantiate(module, {})

print(inst)
print('myAdd', inst.exports['myAdd'](7, 55))
print('mySub', inst.exports['mySub'](7, 2))



from ppci.wasm import instantiate, read_wasm
import demo

with open('demo.wasm', 'rb') as f:
    module = read_wasm(f)

inst = instantiate(module, {})

print(inst)
print('myAdd(0, 55)', inst.exports['myAdd'](0, 55), 'should be:', demo.myAdd(0, 55))
print('myAdd(7, 55)', inst.exports['myAdd'](7, 55), 'should be:', demo.myAdd(7, 55))
print('mySub(7, 2)', inst.exports['mySub'](7, 2), 'should be:', demo.mySub(7, 2))
print('mySub(17, 2)', inst.exports['mySub'](17, 2), 'should be:', demo.mySub(17, 2))
print('myFoo(2.2)', inst.exports['myFoo'](2.2), 'should be:', demo.myFoo(2.2))
print('myBar(7)', inst.exports['myBar'](7), 'should be:', demo.myBar(7))

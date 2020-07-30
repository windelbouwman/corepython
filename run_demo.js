
console.log("Demo time!");


const fs = require('fs');
var wasm_data = fs.readFileSync('./demo.wasm');

var module_ = new WebAssembly.Module(wasm_data);
var module = new WebAssembly.Instance(module_);
console.log("myAdd(7,55) = ", module.exports.myAdd(7, 55));

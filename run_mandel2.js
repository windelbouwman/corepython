
console.log("Demo time!");

function bsp_putc(i) {
    var c = String.fromCharCode(i);
    process.stdout.write(c);
}

const fs = require('fs');
var wasm_data = fs.readFileSync('./mandel2.wasm');

var module_ = new WebAssembly.Module(wasm_data);
var module = new WebAssembly.Instance(module_, {x: {putc: bsp_putc}});
module.exports.mandel();

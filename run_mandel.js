
console.log("Demo time!");

function bsp_putc(i) {
    var c = String.fromCharCode(i);
    if (c === 'n') {
        process.stdout.write('\n');
    } else {
        process.stdout.write(c);
    }
}

const fs = require('fs');
var wasm_data = fs.readFileSync('./mandel.wasm');

var module_ = new WebAssembly.Module(wasm_data);
var module = new WebAssembly.Instance(module_, {x: {putc: bsp_putc}});

console.time('mandel');
module.exports.mandel();
console.timeEnd('mandel');

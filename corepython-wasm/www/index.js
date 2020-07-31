import * as wasm from "corepython-wasm";

// Old simple demo:
// var source = `
// def myAddTwo(a: int, b: int) -> int:
//     return a + b + 2

// def myAddThree(a: int, b: int) -> int:
//     return a + b + 3

// `;

// console.log("Compiling source", source);
// var code = wasm.python_to_wasm(source);
// if (code.length > 0) {

//     console.log("Compilation done & done", code.length, "bytes");
    
//     WebAssembly.instantiate(code)
//     .then(result => {
//         var acc = result.instance.exports.myAddTwo(4, 5);
//         console.log("myAddTwo(4, 5) =", acc);
        
//         var acc = result.instance.exports.myAddThree(1, 2);
//         console.log("myAddThree(1, 2) =", acc);
        
//     })  
// }

var source = `

from x import putc

def mandel():
    # """ Print a mandelbrot fractal to the console """
    w = 50.0
    h = 50.0
    y = 0.0
    while y < h:
        x = 0.0
        while x < w:
            Zr = 0.0
            Zi = 0.0
            Tr = 0.0
            Ti = 0.0
            Cr = 2.0 * x / w - 1.5
            Ci = 2.0 * y / h - 1.0

            i = 0
            while i < 50 and Tr + Ti <= 4.0:
                Zi = 2.0 * Zr * Zi + Ci
                Zr = Tr - Ti + Cr
                Tr = Zr * Zr
                Ti = Zi * Zi
                i = i + 1

            if Tr + Ti <= 4.0:
                putc(ord('*'))
            else:
                putc(ord('_'))

            x = x + 1.0

        putc(10)
        y = y + 1.0

`

var code = wasm.python_to_wasm(source);
if (code.length > 0) {
    console.log("Compilation done & done", code.length, "bytes");

    var out = document.getElementById('output');

    var txt = '';

    function my_putc(c) {
        c = String.fromCharCode(c);
        txt += c;
    }
    
    WebAssembly.instantiate(code, {x: { putc: my_putc}})
    .then(result => {
        result.instance.exports.mandel();
        out.innerText += txt;
    })
}

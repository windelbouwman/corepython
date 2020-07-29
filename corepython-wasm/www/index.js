import * as wasm from "corepython-wasm";

var source = `
def myAddTwo(a: int, b: int) -> int:
    return a + b + 2

def myAddThree(a: int, b: int) -> int:
    return a + b + 3

`;

console.log("Compiling source", source);
var code = wasm.python_to_wasm(source);
if (code.length > 0) {

    console.log("Compilation done & done", code.length, "bytes");
    
    WebAssembly.instantiate(code)
    .then(result => {
        var acc = result.instance.exports.myAddTwo(4, 5);
        console.log("myAddTwo(4, 5) =", acc);
        
        var acc = result.instance.exports.myAddThree(1, 2);
        console.log("myAddThree(1, 2) =", acc);
        
    })
    
}

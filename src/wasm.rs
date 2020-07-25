pub struct WasmModule {
    types: Vec<(Vec<Type>, Vec<Type>)>,
    exports: Vec<Export>,
    functions: Vec<Function>,
}

impl WasmModule {
    pub fn new() -> Self {
        WasmModule {
            types: vec![],
            exports: vec![],
            functions: vec![],
        }
    }

    pub fn add_function(&mut self, name: String, params: Vec<Type>, code: Vec<Instruction>) {
        self.types.push((params, vec![Type::I32]));
        let index = self.functions.len();
        self.exports.push(Export { name, index });
        self.functions.push(Function { code });
    }
}

struct Function {
    code: Vec<Instruction>,
}

struct Export {
    name: String,
    index: usize,
}

struct Writer<W>
where
    W: std::io::Write,
{
    buffer: W,
}

pub enum Type {
    I32,
    I64,
    F32,
    F64,
}

impl<W> Writer<W>
where
    W: std::io::Write,
{
    fn new(buffer: W) -> Self {
        Writer { buffer }
    }

    fn write(&mut self, wasm: WasmModule) -> Result<(), std::io::Error> {
        self.write_header()?;

        self.write_type_section(&wasm)?;
        self.write_func_section(&wasm)?;
        self.write_export_section(&wasm)?;
        self.write_code_section(&wasm)?;

        Ok(())
    }

    fn write_type_section(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        let mut buf: Vec<u8> = vec![];
        let mut w2 = Writer::new(&mut buf);
        w2.write_type_signatures(module)?;
        self.write_section(1, &buf)?;
        Ok(())
    }

    fn write_type_signatures(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        let num_types = module.types.len() as u32;
        self.write_vu32(num_types)?;

        for (params, results) in &module.types {
            self.write_byte(0x60)?;
            self.write_vu32(params.len() as u32)?;
            for param_type in params {
                self.write_type(param_type)?;
            }
            self.write_vu32(results.len() as u32)?;
            for result_type in results {
                self.write_type(result_type)?;
            }
        }

        Ok(())
    }

    fn write_type(&mut self, typ: &Type) -> Result<(), std::io::Error> {
        match typ {
            Type::I32 => self.write_byte(0x7f)?,
            Type::I64 => self.write_byte(0x7e)?,
            Type::F32 => self.write_byte(0x7d)?,
            Type::F64 => self.write_byte(0x7c)?,
        }
        Ok(())
    }

    fn write_func_section(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        let mut buf: Vec<u8> = vec![];
        let mut w2 = Writer::new(&mut buf);
        w2.write_function_protos(module)?;

        self.write_section(3, &buf)?;

        Ok(())
    }

    fn write_function_protos(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        self.write_vu32(module.functions.len() as u32)?;
        for (index, _function) in module.functions.iter().enumerate() {
            self.write_index(index)?;
        }
        Ok(())
    }

    fn write_export_section(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        let mut buf: Vec<u8> = vec![];
        let mut w2 = Writer::new(&mut buf);
        w2.write_exports(module)?;

        self.write_section(7, &buf)?;

        Ok(())
    }

    fn write_exports(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        self.write_vu32(module.exports.len() as u32)?;
        for export in &module.exports {
            self.write_str(&export.name)?;
            let kind = 0; // TODO: for now..
            self.write_byte(kind)?;
            self.write_index(export.index)?;
        }
        Ok(())
    }

    fn write_code_section(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        let mut buf: Vec<u8> = vec![];
        let mut w2 = Writer::new(&mut buf);
        w2.write_function_defs(&module)?;

        self.write_section(10, &buf)?;

        Ok(())
    }

    fn write_function_defs(&mut self, module: &WasmModule) -> Result<(), std::io::Error> {
        self.write_vu32(module.functions.len() as u32)?;

        for function in &module.functions {
            let mut buf: Vec<u8> = vec![];
            let mut w2 = Writer::new(&mut buf);
            w2.write_function_def(function)?;
            self.write_blob(&buf)?;
        }

        Ok(())
    }

    fn write_header(&mut self) -> Result<(), std::io::Error> {
        self.buffer.write(b"\x00asm")?;
        self.write_u32(1)?;
        Ok(())
    }

    fn write_function_def(&mut self, function: &Function) -> Result<(), std::io::Error> {
        // Write locals:
        let num_locals = 0; // TODO: locals
        self.write_vu32(num_locals)?;

        for opcode in &function.code {
            self.write_instruction(opcode)?;
        }

        // Inject end at end:
        self.write_instruction(&Instruction::End)?;

        Ok(())
    }

    fn write_instruction(&mut self, opcode: &Instruction) -> Result<(), std::io::Error> {
        match opcode {
            Instruction::End => self.write_byte(0x0B)?,
            Instruction::LocalGet(index) => {
                self.write_byte(0x20)?;
                self.write_index(*index)?;
            }
            Instruction::Return => {
                self.write_byte(0x0F)?;
            }
            Instruction::I32Const(value) => {
                self.write_byte(0x41)?;
                self.write_vi32(*value)?;
            }
            Instruction::I32Add => {
                self.write_byte(0x6A)?;
            }
            Instruction::I32Sub => {
                self.write_byte(0x6B)?;
            }
            Instruction::I32Mul => {
                self.write_byte(0x6C)?;
            }
        }

        Ok(())
    }

    fn write_index(&mut self, index: usize) -> Result<(), std::io::Error> {
        self.write_vu32(index as u32)?;
        Ok(())
    }

    fn write_section(&mut self, id: u8, buf: &[u8]) -> Result<(), std::io::Error> {
        self.write_vu32(id as u32)?; // section id
        self.write_blob(buf)?;
        Ok(())
    }

    fn write_str(&mut self, text: &str) -> Result<(), std::io::Error> {
        self.write_blob(text.as_bytes())?;
        Ok(())
    }

    fn write_blob(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.write_vu32(buf.len() as u32)?;
        self.buffer.write_all(&buf)?;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<(), std::io::Error> {
        use scroll::{IOwrite, LE};
        self.buffer.iowrite_with(value, LE)?;
        Ok(())
    }

    fn write_vu32(&mut self, value: u32) -> Result<(), std::io::Error> {
        leb128::write::unsigned(&mut self.buffer, value as u64)?;
        Ok(())
    }

    fn write_vi32(&mut self, value: i32) -> Result<(), std::io::Error> {
        leb128::write::signed(&mut self.buffer, value as i64)?;
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), std::io::Error> {
        debug!("Byte: {:02X}", byte);
        self.buffer.write(&[byte])?;
        Ok(())
    }
}

pub fn write_wasm(wasm: WasmModule, filename: &str) -> Result<(), std::io::Error> {
    let buf = std::fs::File::create(filename)?;
    let mut writer = Writer::new(buf);
    writer.write(wasm)?;
    Ok(())
}

#[derive(Debug)]
pub enum Instruction {
    End,
    LocalGet(usize),
    // LocalSet(usize),
    // LocalTee(usize),
    I32Const(i32),
    I32Add,
    I32Sub,
    I32Mul,
    Return,
}

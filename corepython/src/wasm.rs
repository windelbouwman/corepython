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

    pub fn add_function(
        &mut self,
        name: String,
        params: Vec<Type>,
        results: Vec<Type>,
        locals: Vec<Type>,
        code: Vec<Instruction>,
    ) {
        self.types.push((params, results));
        let index = self.functions.len();
        self.exports.push(Export { name, index });
        self.functions.push(Function { locals, code });
    }
}

struct Function {
    code: Vec<Instruction>,
    locals: Vec<Type>,
}

struct Export {
    name: String,
    index: usize,
}

struct Writer<'w, W>
where
    W: std::io::Write,
{
    buffer: &'w mut W,
}

#[derive(Clone, PartialEq)]
pub enum Type {
    I32,
    // I64,
    // F32,
    F64,
}

impl<'w, W> Writer<'w, W>
where
    W: std::io::Write,
{
    fn new(buffer: &'w mut W) -> Self {
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
            // Type::I64 => self.write_byte(0x7e)?,
            // Type::F32 => self.write_byte(0x7d)?,
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
        self.buffer.write_all(b"\x00asm")?;
        self.write_u32(1)?;
        Ok(())
    }

    fn write_function_def(&mut self, function: &Function) -> Result<(), std::io::Error> {
        // Write locals:
        let num_locals = function.locals.len() as u32;
        self.write_vu32(num_locals)?;
        for local_type in &function.locals {
            // TODO: group by local type.
            self.write_vu32(1)?;
            self.write_type(local_type)?;
        }

        // Emit opcodes:
        for opcode in &function.code {
            self.write_instruction(opcode)?;
        }

        // Inject end at end:
        self.write_instruction(&Instruction::End)?;

        Ok(())
    }

    fn write_instruction(&mut self, opcode: &Instruction) -> Result<(), std::io::Error> {
        match opcode {
            // Instruction::Nop => self.write_byte(0x01)?,
            Instruction::Block => {
                self.write_byte(0x02)?;
                self.write_byte(0x40)?;
            }
            Instruction::Loop => {
                self.write_byte(0x03)?;
                self.write_byte(0x40)?;
            }
            Instruction::If => {
                self.write_byte(0x04)?;
                self.write_byte(0x40)?;
            }
            Instruction::Else => self.write_byte(0x05)?,
            Instruction::End => self.write_byte(0x0B)?,
            // Instruction::Br(label) => {
            //     self.write_byte(0x0C)?;
            //     self.write_index(*label)?;
            // }
            Instruction::BrIf(label) => {
                self.write_byte(0x0D)?;
                self.write_index(*label)?;
            }
            Instruction::Call(func) => {
                self.write_byte(0x10)?;
                self.write_index(*func)?;
            }
            Instruction::Drp => {
                self.write_byte(0x1A)?;
            }
            Instruction::LocalGet(index) => {
                self.write_byte(0x20)?;
                self.write_index(*index)?;
            }
            Instruction::LocalSet(index) => {
                self.write_byte(0x21)?;
                self.write_index(*index)?;
            }
            Instruction::Return => {
                self.write_byte(0x0F)?;
            }
            Instruction::I32Const(value) => {
                self.write_byte(0x41)?;
                self.write_vi32(*value)?;
            }
            Instruction::F64Const(value) => {
                self.write_byte(0x44)?;
                self.write_f64(*value)?;
            }
            Instruction::I32Eqz => {
                self.write_byte(0x45)?;
            }
            Instruction::I32Eq => {
                self.write_byte(0x46)?;
            }
            Instruction::I32Ne => {
                self.write_byte(0x47)?;
            }
            Instruction::I32LtS => {
                self.write_byte(0x48)?;
            }
            Instruction::I32GtS => {
                self.write_byte(0x4A)?;
            }
            Instruction::I32LeS => {
                self.write_byte(0x4C)?;
            }
            Instruction::I32GeS => {
                self.write_byte(0x4E)?;
            }

            Instruction::F64Lt => {
                self.write_byte(0x63)?;
            }
            Instruction::F64Gt => {
                self.write_byte(0x64)?;
            }
            Instruction::F64Le => {
                self.write_byte(0x65)?;
            }
            Instruction::F64Ge => {
                self.write_byte(0x66)?;
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
            Instruction::I32DivS => {
                self.write_byte(0x6D)?;
            }

            Instruction::I32And => {
                self.write_byte(0x71)?;
            }
            Instruction::I32Or => {
                self.write_byte(0x72)?;
            }

            Instruction::F64Add => {
                self.write_byte(0xA0)?;
            }
            Instruction::F64Sub => {
                self.write_byte(0xA1)?;
            }
            Instruction::F64Mul => {
                self.write_byte(0xA2)?;
            }
            Instruction::F64Div => {
                self.write_byte(0xA3)?;
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

    fn write_f64(&mut self, value: f64) -> Result<(), std::io::Error> {
        use scroll::{IOwrite, LE};
        self.buffer.iowrite_with(value, LE)?;
        Ok(())
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), std::io::Error> {
        // debug!("Byte: {:02X}", byte);
        self.buffer.write_all(&[byte])?;
        Ok(())
    }
}

pub fn write_wasm<W: std::io::Write>(wasm: WasmModule, buf: &mut W) -> Result<(), std::io::Error> {
    let mut writer = Writer::new(buf);
    writer.write(wasm)?;
    Ok(())
}

#[derive(Debug)]
pub enum Instruction {
    // Nop,
    Block,
    Loop,
    If,
    Else,
    End,
    // Br(usize),
    BrIf(usize),
    Call(usize),
    Drp,
    LocalGet(usize),
    LocalSet(usize),
    // LocalTee(usize),
    I32Const(i32),
    F64Const(f64),
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtS,
    // I32Lt_u,
    I32GtS,
    // I32Gt_u,
    I32LeS,
    // I32Le_u,
    I32GeS,
    // I32Ge_u,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32And,
    I32Or,

    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    Return,
}

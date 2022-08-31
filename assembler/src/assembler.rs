use crate::tokenizer::tokenize_operands;


pub type AssemblyCode = Vec<String>;
pub type ByteCode = Vec<u8>;


pub fn assemble(assembly: AssemblyCode) -> ByteCode {

    let byte_code = ByteCode::new();

    for line in assembly {
        let tokens = tokenize_operands(&line);
        println!("{:#?}", tokens);

        // TODO assemble line
    }

    byte_code
}


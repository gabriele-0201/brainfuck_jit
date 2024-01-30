mod jit;
mod parser;
mod tokenizer;

use anyhow::anyhow;

fn main() -> anyhow::Result<()> {
    let file_path = std::env::args()
        .skip(1)
        .next()
        .ok_or(anyhow!("missing filename"))?;

    let code = std::fs::read_to_string(file_path)?;

    // tokenize the input
    let tokens = tokenizer::tokenize(&code)?;
    println!("{:?}", tokens);

    let instructions = parser::parse(tokens)?;
    println!("{:#?}", instructions);

    // todo; find a way to protect index out of bound
    let mut jit = jit::JIT::new()?;
    jit.execute(instructions);

    Ok(())
}

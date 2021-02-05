use derive_more::From;
use clap::Clap;
use std::{
    fs::File,
    io::{Read, Write, BufReader, LineWriter, stdin, stdout, Stdout, BufRead},
    collections::HashMap,
};

mod lib;

use lib::*;


#[derive(From)]
enum Writer {
    File(LineWriter<File>),
    Stdout(LineWriter<Stdout>),
}
impl Writer {
    fn write_all(&mut self, text: String) -> anyhow::Result<()>{
        match self {
            Writer::File(writer) => writer.write_all(text.as_bytes()),
            Writer::Stdout(writer) => writer.write_all(text.as_bytes()),
        }?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    let mut writer: Writer = if let Some(output_file_path) = &opts.output_file_path {
        let file = File::create(&output_file_path)?;
        LineWriter::new(file).into()
    } else {
        LineWriter::new(stdout()).into()
    };
    if let Some(input_file_path) = &opts.input_file_path {
        let file = File::open(&input_file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            //writer.write_all(substitude(line, &opts))?;
        }
    } else {
        let mut buffer = String::new();
        stdin().read_to_string(&mut buffer)?;
        //writer.write_all(substitude(buffer, &opts))?;
    };
    Ok(())
}

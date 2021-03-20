use model::schema;
use std::process::Command;
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
    println!("cargo:rerun-if-changed=svelte-app");
    println!("cargo:rerun-if-changed=src/model.rs");

    let graphql_schema = schema().as_schema_language();

    let dest_path = Path::new("./svelte-app/schema.graphql");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(graphql_schema.as_bytes()).unwrap();


    Command::new("npm")
        .current_dir("./svelte-app")
        .args(&["install"])
        .status().unwrap();
    Command::new("npm")
        .current_dir("./svelte-app")
        .args(&["run", "build"])
        .status().unwrap();
}

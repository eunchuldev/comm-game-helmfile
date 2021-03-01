use std::process::Command;

fn main() {
    Command::new("npm")
        .current_dir("./svelte-app")
        .args(&["install"])
        .status().unwrap();
    Command::new("npm")
        .current_dir("./svelte-app")
        .args(&["run", "build"])
        .status().unwrap();

    println!("cargo:rerun-if-changed=svelte-app");
}

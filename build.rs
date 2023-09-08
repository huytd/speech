use std::process::Command;

const FRONTEND_PATH: &str = "./frontend";
const PUBLIC_PATH: &str = "./public";

fn main() {
    println!("cargo:rerun-if-changed=frontend");
    Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(FRONTEND_PATH)
        .spawn()
        .expect("Could not build frontend!");
    Command::new("cp")
        .arg("-rf")
        .arg(FRONTEND_PATH.to_owned() + "/lib")
        .arg(PUBLIC_PATH)
        .spawn()
        .expect("Could not copy frontend dependencies!");
}

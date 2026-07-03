use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=ui/window.blp");

    let status = Command::new("blueprint-compiler")
        .args(["compile", "--output", "resources/window.ui", "ui/window.blp"])
        .status()
        .expect("blueprint-compiler не найден в PATH");

    if !status.success() {
        panic!("Ошибка компиляции blueprint");
    }
}
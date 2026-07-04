use std::{fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=ui/window.blp");
    println!("cargo:rerun-if-changed=resources/style.css");
    println!("cargo:rerun-if-changed=resources/window.ui");
    println!("cargo:rerun-if-changed=resources/default_bg.png");

    // Компилируем blueprint
    let status = std::process::Command::new("blueprint-compiler")
        .args(["compile", "--output", "resources/window.ui", "ui/window.blp"])
        .status()
        .expect("blueprint-compiler не найден в PATH");

    if !status.success() {
        panic!("Ошибка компиляции blueprint");
    }

    // Копируем файлы в папку рядом с бинарником
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = Path::new(&out_dir)
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap();
    
    let user_resources = target_dir.join("src").join("resources");
    fs::create_dir_all(&user_resources).expect("Не удалось создать директорию src/resources");
    
    // Копируем все ресурсы
    copy_file("resources/window.ui", &user_resources.join("window.ui"));
    copy_file("resources/style.css", &user_resources.join("style.css"));
    copy_file("resources/default_bg.png", &user_resources.join("default_bg.png"));
}

fn copy_file(source: &str, destination: &Path) {
    if Path::new(source).exists() {
        fs::copy(source, destination)
            .unwrap_or_else(|_| panic!("Не удалось скопировать {} в {}", source, destination.display()));
    }
}
use std::io::Write;
use std::path::PathBuf;

use crate::cli_parser;
use dirs::{cache_dir, home_dir};
use inquire::{Confirm, Select, Text};
use notify::Error;
use std::env::current_dir;
use std::fs::{create_dir_all, File};

const LUWAK_DIR: &str = ".luwak";

pub fn luwak_bin() -> Option<PathBuf> {
    let luwak_bin = home_dir().unwrap().join(LUWAK_DIR).join("bin");
    if !luwak_bin.exists() {
        create_dir_all(&luwak_bin).unwrap();
    }
    Some(luwak_bin)
}

pub fn luwak_module() -> Option<PathBuf> {
    let args = cli_parser::args();
    let luwak_module_local = PathBuf::from(current_dir().unwrap().to_str().unwrap())
        .join(&args.js_script.as_str())
        .parent()
        .unwrap()
        .to_path_buf()
        .join("luwak_modules");
    let luwak_module = if luwak_module_local.exists() {
        luwak_module_local
    } else {
        cache_dir().unwrap().join("luwak").join("modules")
    };
    if !luwak_module.exists() {
        create_dir_all(&luwak_module).unwrap();
    }
    Some(luwak_module)
}

pub fn dump_luwak_module_path() -> Option<PathBuf> {
    let args = cli_parser::args();
    let luwak_module = PathBuf::from(current_dir().unwrap().to_str().unwrap())
        .join(&args.js_script.as_str())
        .parent()
        .unwrap()
        .to_path_buf()
        .join("luwak_modules");
    if !luwak_module.exists() {
        create_dir_all(&luwak_module).unwrap();
    }

    Some(luwak_module)
}

pub fn info() -> Option<String> {
    Some(format!(
        r#"
        Luwak Modules : {}
        Luwak Bin : {}
    "#,
        luwak_module().unwrap().to_str().unwrap(),
        luwak_bin().unwrap().to_str().unwrap()
    ))
}

pub fn init() -> Option<()> {
    let args = cli_parser::args();
    let init_target =
        PathBuf::from(current_dir().unwrap().to_str().unwrap()).join(&args.js_script.as_str());
    if !init_target.exists() {
        create_dir_all(&init_target).unwrap();
    }
    let target = if init_target.is_dir() {
        init_target
    } else {
        init_target.parent().unwrap().to_path_buf()
    };

    let name = Text::new("App name :")
        .with_default(&target.iter().last().unwrap().to_str().unwrap())
        .prompt();

    create_file(
        format!("# {} project!", name.unwrap()).as_str(),
        &target.join("README.md"),
    )
    .unwrap();

    let project_type = Select::new(
        "What's your favorite script?",
        vec!["Javascript", "Typescript"],
    )
    .prompt();

    match project_type.unwrap() {
        "Javascript" => {
            create_file("console.log('Hello')", &target.join("main.js")).unwrap();
        }
        _ => {
            create_file("console.log('Hello')", &target.join("main.ts")).unwrap();
        }
    }

    let luwak_modules = Confirm::new("Do you want to use luwak_module?")
        .with_default(false)
        .with_help_message("All dependencies will be stored in the luwak_module directory.")
        .prompt();
    if luwak_modules.unwrap() {
        create_dir_all(&target.join("luwak_modules")).unwrap();
    }
    Some(())
}

fn create_file(content: &str, path: &PathBuf) -> Result<(), Error> {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let mut file =
        File::create(&path).expect(format!("Unable to create {} file", file_name).as_str());
    file.write_all(content.as_bytes())
        .expect(format!("Unable to create {} file", file_name).as_str());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_luwakdir() {
        assert_eq!(Some(home_dir().unwrap().join(LUWAK_DIR)), luwak_bin())
    }
}

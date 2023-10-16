use std::path::PathBuf;

use dirs::home_dir;
use std::env::current_dir;
use std::fs::create_dir_all;

const LUWAK_DIR: &str = ".luwak";

pub fn luwak_bin() -> Option<PathBuf> {
    let luwak_bin = home_dir().unwrap().join(LUWAK_DIR).join("bin");
    if !luwak_bin.exists() {
        create_dir_all(&luwak_bin).unwrap();
    }
    Some(luwak_bin)
}

pub fn luwak_module() -> Option<PathBuf> {
    let luwak_module_local = current_dir().unwrap().join("luwak_modules");
    let luwak_module = if luwak_module_local.exists() {
        luwak_module_local
    } else {
        home_dir().unwrap().join(LUWAK_DIR).join("modules")
    };
    if !luwak_module.exists() {
        create_dir_all(&luwak_module).unwrap();
    }
    Some(luwak_module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_luwakdir() {
        assert_eq!(Some(home_dir().unwrap().join(LUWAK_DIR)), luwak_bin())
    }
}

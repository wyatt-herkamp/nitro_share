use std::path::PathBuf;

pub fn push_to_extension(path: &PathBuf, extension: &str) -> PathBuf {
    let mut path = path.to_path_buf();
    if let Some(value) = path.extension() {
        let mut os_string = value.to_os_string();
        os_string.push(format!(".{}", extension));
        path.set_extension(os_string);
    } else {
        path.with_extension(extension);
    }
    path
}

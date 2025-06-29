#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

pub mod error;   pub use error::{ Error, Result };
pub mod prelude;

pub mod manager;  pub use manager::TaskManager;
pub mod session;  pub use session::{ Session, Tab };

/// Generates new path
pub fn new_path<P: AsRef<std::path::Path>>(path: P) -> Result<std::path::PathBuf> {
    let path = path.as_ref().to_path_buf();

    if path.to_string_lossy().starts_with("/") {
        root_path(&path)
    } else {
        Ok(path)
    }
}

/// Generates path by program root path 
pub fn root_path<P: AsRef<std::path::Path>>(relative_path: P) -> Result<std::path::PathBuf> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| Error::InvalidRootPath)?;

    // converting path to string:
    let rel_str = relative_path.as_ref().to_str().ok_or_else(|| Error::InvalidPath)?;
    
    // removing start symbol '/' if it's exists
    let rel_str = if rel_str.starts_with('/') || rel_str.starts_with('\\') {
        &rel_str[1..]
    } else {
        rel_str
    };

    Ok(exe_dir.join(rel_str.replace("/", "\\")))
}

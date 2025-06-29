use macron::{ Display, Error, From };

// Result alias
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + std::marker::Send + std::marker::Sync + 'static>>;

// Chromedriver API Error
#[derive(Debug, Display, Error, From)]
pub enum Error {
    #[display = "Couldn't get the directory of the root path"]
    InvalidRootPath,

    #[display = "The path contains invalid UTF-8 characters"]
    InvalidPath,
    
    #[display = "Incorrect session ID"]
    IncorrectSessionId,

    #[display = "Incorrect window handle"]
    IncorrectWindowHandle,

    #[display = "Incorrect window handles list"]
    IncorrectWindowHandles,

    #[display = "No window handles found"]
    NoWindowHandles,

    #[display = "Failed to connect to CDP (Chrome DevTools Protocol)"]
    CdpConnectionFailed,

    #[display = "CDP command execution failed"]
    CdpCommandFailed,

    #[display = "Element not found for the given selector"]
    ElementNotFound
}

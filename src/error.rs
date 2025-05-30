use macron::{ Display, Error, From };

// Result alias
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Chromedriver API Error
#[derive(Debug, Display, Error, From)]
pub enum Error {
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

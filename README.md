[![github]](https://github.com/fuderis/rs-chromedriver-api)&ensp;
[![crates-io]](https://crates.io/crates/chromedriver-api)&ensp;
[![docs-rs]](https://docs.rs/chromedriver-api)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# Chromedriver API (auto clicker)

## Introduction:

This API is designed for interacting with chromedriver (browser auto clicker).


## Basic Methods:

* **.open(URL)** - Open URL-address on Chrome browser window
* **.inject(JS_SCRIPT)** - Inject JavaScript code to process
* **.click(CSS_SELECTOR)** - Click to element by CSS selector
* **.value(CSS_SELECTOR, VALUE)** - Change element value by CSS selector


## Examples:

```rust
use chromedriver_api::{ Result, Session };
use tokio::time::{ sleep, Duration };

#[tokio::main]
async fn main() -> Result<()> {
    // running chromedriver session:
    let mut session = Session::run("54477", Some("C:\\Users\\Synap\\AppData\\Local\\Google\\Chrome\\Profiles\\Profile1")).await?;

    println!("[INFO]: the session is launched on port [54477] ..");

    // opening website URL:
    session.open("https://vk.com/otaku_lounge?w=wall-230618027_38").await?;

    println!("[INFO]: loaded page url 'https://vk.com/otaku_lounge?w=wall-230618027_38'");


    sleep(Duration::from_secs(1)).await;  // (just waiting some times for test)
    
    // executing script:
    match script(&session).await {
        Ok(_) => println!("[INFO] Script is successfully executed!"),
        Err(e) => eprintln!("[INFO] Executing script error: {e}"),
    }
    
    // executing our scripts:
    sleep(Duration::from_secs(2)).await;  // (just waiting some times for test)
    

    // closing session && stopping server:
    session.close().await?;
    println!("[INFO]: the session is closed");

    Ok(())
}

async fn script(session: &Session) -> Result<()> {
    // click to button:
    session.click(r#"div[aria-label="Отправить реакцию «Лайк»"]"#).await?;

    Ok(())
}
```

## Licensing:

Distributed under the MIT license.


## Feedback:

You can contact me via GitHub or send a message to my Telegram [@fuderis](https://t.me/fuderis).

This library is constantly evolving, and I welcome your suggestions and feedback.

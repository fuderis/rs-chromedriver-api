[![github]](https://github.com/fuderis/rs-chromedriver-api)&ensp;
[![crates-io]](https://crates.io/crates/chromedriver-api)&ensp;
[![docs-rs]](https://docs.rs/chromedriver-api)

[github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
[docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs

# Chromedriver API (auto clicker)

## Introduction:

This API is designed for interacting with chromedriver (browser auto clicker).


## Examples:

```rust
use chromedriver_api::{ Result, Session };
use tokio::time::{ sleep, Duration };

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = Session::run("54477", Some("C:\\Users\\Admin\\AppData\\Local\\Google\\Chrome\\Profiles\\Profile1")).await?;
    println!("[INFO]: the session is launched on port [54477] ..");

    // open frist tab:
    let first_tab = session.open("https://example.com/").await?;
    let mut first_tab = first_tab.lock().await;
    println!("[INFO]: a new tab is opened on 'https://example.com/' ..");
    
    sleep(Duration::from_secs(1)).await;

    // open second tab:
    let second_tab = session.open("https://example.com/").await?;
    let mut second_tab = second_tab.lock().await;
    println!("[INFO]: a new tab is opened on 'https://example.com/' ..");

    sleep(Duration::from_secs(1)).await;

    // inject script to first tab:
    first_tab.inject(r#"
        alert("Ok!")
    "#).await?;

    sleep(Duration::from_secs(1)).await;

    // do second tab active:
    second_tab.active().await?;

    sleep(Duration::from_secs(1)).await;

    // close second tab:
    second_tab.close().await?;
    println!("[INFO]: the second tab is closed");

    sleep(Duration::from_secs(1)).await;

    // close session:
    session.close().await?;
    println!("[INFO]: the session is closed");

    Ok(())
}

```

## Licensing:

Distributed under the MIT license.


## Feedback:

You can contact me via GitHub or send a message to my Telegram [@fuderis](https://t.me/fuderis).

This library is constantly evolving, and I welcome your suggestions and feedback.

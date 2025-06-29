extern crate chromedriver_api;  use chromedriver_api::{ prelude::*, Session };

use tokio::time::{ sleep, Duration };

#[tokio::main]
async fn main() -> Result<()> {
    let mut session = Session::run("54477", Some("/bin/chromedriver/chromedriver.exe"), Some("C:/Users/Synap/AppData/Local/Google/Chrome/Profiles/Profile1"), false).await?;
    println!("[INFO]: the session is launched on port [54477] ..");

    // open first tab:
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

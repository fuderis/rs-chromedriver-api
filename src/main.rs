use chromedriver_api::{ prelude::*, Session };
use tokio::time::{ sleep, Duration };
use macron::path;

#[tokio::main]
async fn main() -> Result<()> {
    let free_port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port().to_string();
    let chrome_path = path!("bin/chromedriver/chromedriver.exe");
    let session_path = path!("%/ChromeDriver/Profile");

    let mut session = Session::run(
        &free_port,                           // server port
        chrome_path,       // path to binary chromedriver file
        Some(session_path),     // directory to save/load chrome session
        false                       // headless mode on/off
    ).await?;
    println!("[INFO]: the session is launched on port [{free_port}] ..");


    // open first tab:
    let first_tab = session.open("https://example.com/").await?;
    let mut first_tab = first_tab.lock().await;
    println!("[INFO]: a new tab is opened on 'https://example.com/' ..");
    
    // inject script to first tab:
    // first_tab.inject::<()>(r#"
    //     alert("Ok!")
    // "#).await?;

    sleep(Duration::from_secs(1)).await;

    // close first tab:
    println!("[INFO]: the first tab is closed");
    first_tab.close().await?;


    // open second tab:
    let second_tab = session.open("https://example.com/").await?;
    let mut second_tab = second_tab.lock().await;
    println!("[INFO]: a new tab is opened on 'https://example.com/' ..");

    sleep(Duration::from_secs(1)).await;

    // close second tab:
    println!("[INFO]: the second tab is closed");
    second_tab.close().await?;
    sleep(Duration::from_secs(1)).await;


    // close session:
    session.close().await?;
    println!("[INFO]: the session is closed");

    Ok(())
}

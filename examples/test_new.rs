use chromedriver_api::{ prelude::*, Session };
use tokio::time::{ sleep, Duration };
use macron::path;

#[tokio::main]
async fn main() -> Result<()> {
    // run chromedriver:
    let free_port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let chrome_path = path!("bin/chromedriver/chromedriver.exe");
    let session_path = path!("%/ChromeDriver/Profile");

    let session = Session::run(
        free_port,                  
        chrome_path,     
        Some(session_path),   
        false                
    ).await?;

    // get session id & remove session handler (for tests):
    let session_id = session.get_id().to_string();
    drop(session);

    sleep(Duration::from_secs(2)).await;

    // re-create chromedriver handler:
    let session = Session::new(free_port, session_id).await?;
    session.close().await?;

    Ok(())
}

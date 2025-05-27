extern crate chromedriver_api;  use chromedriver_api::{ Result, Session };

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

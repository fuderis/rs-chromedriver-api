use chromedriver_api::{ prelude::*, Session };
use tokio::time::{ sleep, Duration };
use macron::path;

#[tokio::main]
async fn main() -> Result<()> {
    let free_port = std::net::TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    let chrome_path = path!("bin/chromedriver/chromedriver.exe");
    let session_path = path!("%/ChromeDriver/Profile");

    let session = Session::run(
        free_port, 
        chrome_path,
        Some(session_path),
        false   // headless mode
    ).await?;
    println!("[INFO]: session launched on port [{free_port}]");

    // Tab 1: Normal page (fast close test)
    let tab1 = session.open("https://example.com").await?;
    let tab1 = tab1.lock().await;
    println!("[INFO]: tab1: form page loaded");

    sleep(Duration::from_secs(2)).await;

    // Tab 2: Page with beforeunload handler (blocks close)
    let tab2 = session.open("https://html-online.com/editor/").await?;
    let tab2 = tab2.lock().await;
    tab2.inject::<()>(r#"
        window.addEventListener('beforeunload', function(e) {
            e.preventDefault();
            e.returnValue = 'Are you sure?';
            return 'Are you sure?';
        });
    "#).await?;
    println!("[INFO]: tab2: beforeunload hook installed");

    sleep(Duration::from_secs(2)).await;

    // Tab 3: Alert + Confirm scenario (multiple retries)
    let tab3 = session.open("https://httpbin.org/html").await?;
    let tab3 = tab3.lock().await;
    tab3.inject::<()>(r#"
        // Delayed alert 3s after close attempt
        setTimeout(() => {
            alert('Late alert!');
        }, 3000);
        
        // Confirm after 1s
        setTimeout(() => {
            if (confirm('Close tab?')) {
                console.log('User confirmed');
            }
        }, 1000);
    "#).await?;
    println!("[INFO]: tab3: delayed alert + confirm injected");

    sleep(Duration::from_secs(2)).await;

    println!("\n[TEST]: Closing tab1 (should be ✅ fast)");
    tab1.close().await?;
    println!("[✅] tab1 closed successfully\n");

    println!("[TEST]: Closing tab2 (should trigger ⚠️ beforeunload retry)");
    tab2.close().await?;
    println!("[✅] tab2 closed (retry worked)\n");

    println!("[TEST]: Closing tab3 (should trigger ⚠️ alert/confirm retries)");
    tab3.close().await?;
    println!("[✅] tab3 closed (multiple retries succeeded)\n");

    // Verify remaining handles
    let handles = session.get_tabs_ids().await?;
    println!("[INFO]: Remaining tabs: {}", handles.len());

    sleep(Duration::from_secs(1)).await;
    
    session.close().await?;
    println!("[INFO]: Session closed");

    Ok(())
}

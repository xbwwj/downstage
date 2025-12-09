use std::time::Duration;

use downstage::browser::Browser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    {
        let browser = Browser::launch().await.unwrap();
        {
            let _page = browser.new_page().await;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        println!("page should close");

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
    println!("browser should close");

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

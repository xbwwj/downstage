use std::time::Duration;

use downstage::browser::Browser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let browser = Browser::launch().await?;
    let page = browser.new_page().await?;

    page.goto("https://rust-lang.org/").await?;
    let element = page.query_selector(".button-download").await?;
    dbg!(element);

    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(())
}

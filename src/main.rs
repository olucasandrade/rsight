use rsight::search;

#[tokio::main]
async fn main() {
    // CLI smoke test: search for "main" in $HOME, print first 5 results
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut rx = search(&home, "main").await;
    let mut count = 0;
    while let Some(result) = rx.recv().await {
        println!("{:?}", result);
        count += 1;
        if count >= 5 { break; }
    }
}

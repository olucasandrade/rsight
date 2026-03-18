use rsight::search;

#[tokio::main]
async fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let query = std::env::args().nth(1).unwrap_or_else(|| "main".into());

    println!("Searching {} for {:?}...", home, query);
    let mut rx = search(&home, &query).await;
    let mut count = 0;
    while let Some(result) = rx.recv().await {
        println!("{:?}", result);
        count += 1;
        if count >= 10 { break; }
    }
    println!("Done. {} results shown (max 10).", count);
}

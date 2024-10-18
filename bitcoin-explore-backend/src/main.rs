use reqwest::Error;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{self, Duration};
use warp::Filter;

#[derive(Deserialize)]
struct BtcPrice {
    bitcoin: CurrencyPrice,
}

#[derive(Deserialize)]
struct CurrencyPrice {
    usd: f64,
}

#[derive(Serialize)]
struct Metrics {
    block_height: u64,
    btc_price: f64,
    timestamp: String,
}

async fn fetch_block_height() -> Result<u64, Error> {
    let url = "https://blockstream.info/api/blocks/tip/height";
    let response = reqwest::get(url).await?.json::<u64>().await?;
    Ok(response)
}

async fn fetch_btc_price() -> Result<f64, Error> {
    let url = "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
    let response: BtcPrice = reqwest::get(url).await?.json().await?;
    Ok(response.bitcoin.usd)
}

fn create_metrics_table(conn: &Connection) -> Result<()> {
    // Create table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY,
            block_height INTEGER,
            btc_price REAL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    Ok(())
}

fn save_metrics(conn: &Connection, block_height: u64, btc_price: f64) -> Result<()> {
    conn.execute(
        "INSERT INTO metrics (block_height, btc_price, timestamp) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        params![block_height, btc_price],
    )?;

    Ok(())
}

fn get_metrics_history(conn: &Connection) -> Result<Vec<Metrics>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT block_height, btc_price, timestamp FROM metrics ORDER BY id DESC LIMIT 50")?;

    let metrics_iter = stmt.query_map([], |row| {
        Ok(Metrics {
            block_height: row.get(0)?,
            btc_price: row.get(1)?,
            timestamp: row.get(2)?,
        })
    })?;

    let mut metrics = Vec::new();
    for metric in metrics_iter {
        metrics.push(metric?);
    }

    Ok(metrics)
}

fn create_metrics_route(
    conn: Arc<Mutex<Connection>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("api" / "metrics")
        .and(warp::get())
        .map(move || {
            let metrics = {
                // Handle poisoned lock gracefully
                let conn = match conn.lock() {
                    Ok(c) => c,
                    Err(poisoned) => {
                        eprintln!("Mutex poisoned, recovering: {:?}", poisoned);
                        poisoned.into_inner()
                    }
                };

                match get_metrics_history(&conn) {
                    Ok(metrics) => metrics,
                    Err(e) => {
                        eprintln!("Error fetching metrics history: {}", e);
                        vec![]
                    }
                }
            };

            warp::reply::json(&metrics)
        })
}

#[tokio::main]
async fn main() {
    println!("Starting backend...");

    let conn = Arc::new(Mutex::new(Connection::open("metrics.db").expect("Failed to open database")));

    // Create the metrics table at startup if it doesn't exist
    {
        let conn = conn.lock().unwrap();
        if let Err(e) = create_metrics_table(&conn) {
            eprintln!("Error creating metrics table: {}", e);
        }
    }

    let conn_for_route = Arc::clone(&conn);

    // Create the metrics route with CORS enabled
    let metrics_route = create_metrics_route(conn_for_route);

    // Enable CORS for the API
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["content-type"]);

    // Start the warp server
    tokio::spawn(async move {
        println!("Starting the Warp server on port 8080...");
        warp::serve(metrics_route.with(cors))
            .run(([0, 0, 0, 0], 8080))
            .await;
    });

    let mut interval = time::interval(Duration::from_secs(20));

    loop {
        interval.tick().await;

        match (fetch_block_height().await, fetch_btc_price().await) {
            (Ok(block_height), Ok(btc_price)) => {
                println!("Fetched block height and BTC price: {}, {}", block_height, btc_price);

                let conn = match conn.lock() {
                    Ok(c) => c,
                    Err(poisoned) => {
                        eprintln!("Mutex poisoned, recovering: {:?}", poisoned);
                        poisoned.into_inner()
                    }
                };

                if let Err(e) = save_metrics(&conn, block_height, btc_price) {
                    eprintln!("Error saving metrics: {}", e);
                }
            }
            (Err(e), _) => eprintln!("Error fetching block height: {}", e),
            (_, Err(e)) => eprintln!("Error fetching BTC price: {}", e),
        }
    }
}

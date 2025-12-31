use kiwi_store::{Store, Key, Value};
use std::fs::{self, File};
use std::io::{BufReader, BufRead};
use std::time::Instant;
use anyhow::{anyhow, Result};
fn main() -> Result<()> {
    let examples_dir = std::env::current_dir()?.join("examples");

    let pageview_file = fs::read_dir(&examples_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file() &&
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("pageviews-"))
                    .unwrap_or(false)
        });

    let file_path = match pageview_file {
        Some(path) => path,
        None => {
            eprintln!("Error: No pageview file found in examples/ directory!");
            eprintln!("\nPlease download and extract a Wikipedia pageview file:");
            return Err(anyhow!("No pageview file found"));
        }
    };

    println!("Processing: {}", file_path.file_name().unwrap().to_str().unwrap());

    let store_path = "/Users/codex/RustroverProjects/kiwi-store-template-patel-vedant/wikipedia_pageviews_store";
    let mut store = Store::with_path(store_path)?;

    let start = Instant::now();
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let mut line_count = 0;

    for result in reader.lines() {
        let line = result?;
        line_count += 1;

        // ========== CODE FROM PDF STARTS HERE ==========
        // Parse each line: <domain_code> <page_name> <view_count> ...
        let mut tokens = line.split_ascii_whitespace();
        let domain_code = tokens.next().ok_or(anyhow!("Invalid CSV line"))?;
        let page_name = tokens.next().ok_or(anyhow!("Invalid CSV line"))?;
        let view_count_str = tokens.next().ok_or(anyhow!("Invalid CSV line"))?;
        let view_count = view_count_str.parse::<i64>()?;
        let key = format!("{}/{}", domain_code, page_name);
        store.put(Key::String(key), Value::Int(view_count));
        // ========== CODE FROM PDF ENDS HERE ==========

        if line_count % 100_000 == 0 {
            println!("Processed {} lines...", line_count);
        }
    }

    let elapsed = start.elapsed();

    println!("\n=== Summary ===");
    println!("Lines processed: {}", line_count);
    println!("Time: {:.2}s", elapsed.as_secs_f64());
    println!("Throughput: {:.0} lines/sec", line_count as f64 / elapsed.as_secs_f64());

    let frag_ratio = store.fragmentation_ratio();
    println!("Fragmentation: {:.2}%", frag_ratio * 100.0);

    if frag_ratio > 0.3 {
        println!("\nRunning compaction...");
        let compact_start = Instant::now();
        let bytes_reclaimed = store.compact()?;
        let compact_elapsed = compact_start.elapsed();
        println!("Compacted in {:.2}s", compact_elapsed.as_secs_f64());
        println!("Reclaimed: {:.2} MB", bytes_reclaimed as f64 / 1_048_576.0);
    }

    println!("\nStore saved to: {}", store_path);

    Ok(())
}
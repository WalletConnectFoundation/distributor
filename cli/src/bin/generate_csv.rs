use clap::{Arg, Command};
use csv::Writer;
use solana_sdk::signature::{Keypair, Signer};
use std::fs::File;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("CSV Generator")
        .version("1.0")
        .about("Generates CSV with valid Solana wallet addresses for testing")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output CSV file path")
                .default_value("test_recipients.csv"),
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("NUMBER")
                .help("Number of addresses to generate")
                .default_value("30000"),
        )
        .arg(
            Arg::new("amount")
                .short('a')
                .long("amount")
                .value_name("TOKENS")
                .help("Token amount per recipient")
                .default_value("1000"),
        )
        .arg(
            Arg::new("locked")
                .short('l')
                .long("locked")
                .value_name("TOKENS")
                .help("Locked token amount per recipient")
                .default_value("0"),
        )

        .get_matches();

    let output_file = matches.get_one::<String>("output").unwrap();
    let count: usize = matches.get_one::<String>("count").unwrap().parse()?;
    let amount = matches.get_one::<String>("amount").unwrap();
    let locked = matches.get_one::<String>("locked").unwrap();

    println!("Generating {} wallet addresses...", count);
    println!("Output file: {}", output_file);

    let file = File::create(output_file)?;
    let mut wtr = Writer::from_writer(file);

    // Create private keys file for debugging
    let keys_filename = output_file.replace(".csv", "_private_keys.txt");
    println!("Private keys will be saved to: {}", keys_filename);
    let mut keys_file = File::create(keys_filename)?;

    // Write CSV header
    wtr.write_record(&["pubkey", "amount", "locked_amount"])?;

    // Write private keys file header
    writeln!(keys_file, "# Private Keys for Debugging - KEEP SECURE!")?;
    writeln!(keys_file, "# Format: pubkey,private_key_base58")?;
    writeln!(keys_file, "pubkey,private_key")?;

    let start_time = std::time::Instant::now();

    // Generate addresses in batches for progress reporting
    const BATCH_SIZE: usize = 1000;
    for batch in 0..(count + BATCH_SIZE - 1) / BATCH_SIZE {
        let batch_start = batch * BATCH_SIZE;
        let batch_end = std::cmp::min(batch_start + BATCH_SIZE, count);

        for i in batch_start..batch_end {
            // Generate a new keypair
            let keypair = Keypair::new();
            let pubkey = keypair.pubkey().to_string();

            // Write CSV record
            wtr.write_record(&[&pubkey, amount, locked])?;

            // Write private key for debugging
            let private_key_bytes = keypair.to_bytes();
            let private_key_base58 = bs58::encode(&private_key_bytes).into_string();
            writeln!(keys_file, "{},{}", pubkey, private_key_base58)?;
        }

        // Progress update
        let progress = ((batch_end as f64 / count as f64) * 100.0) as u32;
        println!("Progress: {}% ({}/{})", progress, batch_end, count);
    }

    wtr.flush()?;
    keys_file.flush()?;

    let duration = start_time.elapsed();
    println!(
        "✅ Generated {} addresses in {:.2} seconds",
        count,
        duration.as_secs_f64()
    );
    println!("📄 CSV saved to: {}", output_file);
    
    let keys_filename = output_file.replace(".csv", "_private_keys.txt");
    println!("🔑 Private keys saved to: {}", keys_filename);
    println!("⚠️  WARNING: Keep the private keys file secure and delete it when no longer needed!");

    Ok(())
}

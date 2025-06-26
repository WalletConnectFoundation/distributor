# Upload Merkle Tree to PostgreSQL

This document describes how to use the optimized `upload-merkle-tree` command to efficiently upload merkle tree data from JSON files to a PostgreSQL database.

## Prerequisites

1. **PostgreSQL Database**: Ensure you have access to a PostgreSQL database
2. **Database Setup**: The script will automatically create the necessary tables if they don't exist
3. **Dependencies**: The required Rust dependencies have been added to `cli/Cargo.toml`
4. **Merkle Distributor Parameters**: You need the program ID, base key, and mint address for your merkle distributor

## Database Schema

The script expects an existing table with the following structure:

```sql
CREATE TABLE IF NOT EXISTS "airdrop_recipients" (
  "uuid" uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
  "index" varchar(255) NOT NULL,
  "recipient" varchar(255) NOT NULL,
  "amount" varchar(255) NOT NULL,
  "proof" text[] NOT NULL
);
```

Where:
- `index`: The **merkle distributor PDA** encoded as base58 (same for all records in one upload)
- `recipient`: The claimant's public key encoded as base58
- `amount`: The claimable amount encoded as a hex string (e.g., "0x1a2b3c")
- `proof`: Array of merkle proof elements as hex strings

## Usage

### Basic Usage

```bash
cargo run --bin cli upload-merkle-tree \
    --merkle-tree-path ./merkle_trees_devnet/ \
    --postgres-url "host=localhost user=postgres dbname=airdrop_db password=your_password" \
    --program-id YOUR_PROGRAM_ID \
    --base YOUR_BASE_KEY \
    --mint YOUR_MINT_ADDRESS
```

### Parameters

#### Required Parameters:
- `--merkle-tree-path`: Path to the folder containing merkle tree JSON files
- `--postgres-url`: PostgreSQL connection string
- `--program-id`: Merkle distributor program ID
- `--base`: Base key for the merkle distributor
- `--mint`: Token mint address

#### Optional Parameters:
- `--table-name`: Name of the table to store data (default: "airdrop_recipients")

## Environment Variables

You can also set the parameters using environment variables:

```bash
export MERKLE_TREE_PATH="./merkle_trees_devnet/"
export POSTGRES_URL="<connection_string>"
export TABLE_NAME="airdrop_recipients"
export PROGRAM_ID="<deployed_program_id>" # BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy
export BASE=$(solana-keygen pubkey keys/base-devnet.json)
export MINT=$(cat token_mint.txt)

cargo run --bin cli upload-merkle-tree
```
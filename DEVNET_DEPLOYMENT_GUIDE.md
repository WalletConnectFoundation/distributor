# Jupiter Distributor: Devnet Deployment Guide for 30K Recipients

## Prerequisites

### System Requirements

- **Rust**: 1.68.0+
- **Solana CLI**: 1.16.25+
- **Anchor**: 0.28.0+
- **Node.js**: 16+
- **Yarn**: 1.22.22+

### Initial Setup

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.16.25/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.28.0
avm use 0.28.0

# Verify installations
solana --version
anchor --version
```

## Phase 1: Environment Setup

### 1. Configure Solana for Devnet

```bash
# Set cluster to devnet
solana config set --url https://api.devnet.solana.com

# Generate or import your admin keypair
solana-keygen new --outfile keys/admin-devnet.json

# Fund your admin account (you'll need ~100 SOL for deployment + token funding)
solana airdrop 2 $(solana-keygen pubkey keys/admin-devnet.json)
# Repeat airdrop command multiple times to get sufficient funds
```

### 2. Build the Project

```bash
# Build the Anchor program
anchor build -p merkle_distributor

# Build the CLI tool
cargo build -p jup-scripts --release
```

### 3. Deploy Program to Devnet (Optional)

If you need to deploy your own instance:

```bash
# Update Anchor.toml cluster to devnet
anchor deploy --provider.cluster devnet
```

## Phase 2: Token Setup

### 4. Create or Use Existing Token

```bash
# Option A: Create a new token
spl-token create-token --decimals 6 > token_mint.txt
export TOKEN_MINT=$(cat token_mint.txt)

# Option B: Use existing token (replace with your token mint)
export TOKEN_MINT="YourTokenMintAddressHere"

# Create token account for admin (skip if already exists)
# Check existing accounts first: spl-token accounts --url https://api.devnet.solana.com
spl-token create-account $TOKEN_MINT --url https://api.devnet.solana.com

# Mint tokens (30k recipients × 1000 tokens each = 30M tokens + buffer)
# Adjust amount based on your token distribution plan
spl-token mint $TOKEN_MINT 35000000 # 35M tokens for safety buffer
```

## Phase 3: Recipient Data Preparation

### 5. Prepare Recipients CSV

**Option A: Fast Rust Generator (Recommended)**

```bash
# Build the CSV generator
cargo build --release --bin generate_csv

# Generate 30K test addresses (takes ~0.5 seconds!)
./target/release/generate_csv --count 30000 --output recipients_30k.csv

# Custom amounts and locked tokens
./target/release/generate_csv --count 30000 --amount 1500 --locked 500 --output recipients_custom.csv
```

**Option B: Manual CSV Creation**

Create a file called `recipients_30k.csv` with the following format:

```csv
pubkey,amount,locked_amount
wallet_address_1,1000,0
wallet_address_2,1500,500
wallet_address_3,2000,0
```

**Important**: Ensure all wallet addresses are valid Solana public keys and amounts are specified in token units (not lamports).

### 6. Validate CSV Data

```bash
# Check CSV format and validate addresses
head recipients_30k.csv
wc -l recipients_30k.csv  # Should show 30,001 lines (including header)

# Optional: Create test script to validate all addresses
```

## Phase 4: Merkle Tree Generation

### 7. Choose Your Airdrop Configuration

Before setting variables, understand your options:

#### **Important: Security Model**

🔒 **ALL claim types require being in the merkle tree** - this is your security whitelist!
- Only wallets in your `recipients_30k.csv` can claim
- They must provide a valid merkle proof
- The blockchain verifies the proof automatically

The "approval" setting controls **additional** approval beyond the merkle tree:

#### **Claim Types (Choose One):**

**Option A: Simple Immediate Airdrop (Recommended for your use case)**
- Recipients in the merkle tree can claim directly
- No additional admin approval needed per claim
- Tokens usable right away
- Perfect for community airdrops

**Option B: Operator-Controlled Airdrop**
- Recipients must be in merkle tree AND admin must sign each claim
- Double approval system (merkle proof + admin signature)
- Good for controlled distributions where you want manual review
- More complex setup

**Option C: Staking-Integrated Airdrop**
- Claimed tokens go directly to staking
- Requires staking infrastructure
- Advanced use case

#### **Practical Example:**

**Your CSV has 30K wallet addresses → Only those 30K wallets can ever claim**

- **Option A (Permissionless)**: Any of the 30K wallets can claim anytime without asking you
- **Option B (Permissioned)**: The 30K wallets can only claim when you co-sign their transaction
- **Random wallet not in CSV**: Can NEVER claim regardless of option chosen

**For your use case (simple community airdrop), choose Option A.**

#### **Vesting Options:**

**Immediate (Minimal Vesting) - For Simple Airdrops:**
- Tokens claimable with minimal delay (1-2 minutes)
- Effectively immediate claiming
- What you want for "claim and use" scenario
- Note: Program requires future timestamps, so true "immediate" isn't possible

**Linear Vesting:**
- Tokens unlock gradually over time
- Good for team/investor distributions

#### **Summary for Your Use Case:**

Since you want a **simple, immediate airdrop** where recipients can claim and use tokens right away:

✅ **Use OPTION A in the configuration below:**
- **CLAIM_TYPE=0**: Permissionless (no approval needed)
- **START_VESTING_TS**: 1 minute in future (minimal delay)
- **END_VESTING_TS**: 2 minutes in future (short vesting period)
- **CLAWBACK_START_TS**: 1 day later (time to reclaim unclaimed tokens)

This gives you:
- 📤 **Near-immediate claiming**: Recipients can claim 1 minute after deployment
- 🚀 **Minimal waiting**: Tokens fully vested within 2 minutes
- 🔄 **Clawback**: You can reclaim unclaimed tokens after 1 day
- 🎯 **Simple**: No complex approvals or staking required
- ⚠️ **Program limitation**: True instant claiming isn't possible due to timestamp validation

### 8. Set Environment Variables

```bash
# Configuration
export CSV_PATH="./recipients_30k.csv"
export MERKLE_TREE_PATH="./merkle_trees_devnet"
export TOKEN_DECIMALS="6"  # Adjust based on your token
export BASE_PATH="keys/base-devnet.json"
export KEYPAIR_PATH="keys/admin-devnet.json"
export RPC="https://api.devnet.solana.com"
export PRIORITY_FEE="1000000"  # 0.001 SOL priority fee

# Generate base keypair (used for deterministic distributor addresses)
solana-keygen new --outfile $BASE_PATH

# Timing configuration (Unix timestamps)
export CURRENT_TIME=$(date +%s)

# OPTION A: Immediate Claiming (Minimal Vesting) - RECOMMENDED FOR YOUR USE CASE
# NOTE: Program requires ALL timestamps to be in the future, so we use minimal vesting periods
export START_VESTING_TS=$((CURRENT_TIME + 60))       # 1 minute from now (tokens start unlocking)
export END_VESTING_TS=$((CURRENT_TIME + 120))        # 2 minutes from now (tokens fully unlocked)
export CLAWBACK_START_TS=$((END_VESTING_TS + 86400)) # 1 day after vesting ends (for unclaimed tokens)

# OPTION B: Vesting Airdrop (Uncomment if you want gradual unlock)
# export START_VESTING_TS=$((CURRENT_TIME + 60))       # 1 minute from now (vesting starts)
# export END_VESTING_TS=$((CURRENT_TIME + 1200))       # 20 minutes from now (vesting ends)
# export CLAWBACK_START_TS=$((END_VESTING_TS + 86400)) # 1 day after vesting ends

# Get activation slot (tokens claimable immediately)
export ACTIVATION_POINT=$(solana slot --url $RPC)
export ACTIVATION_TYPE=0  # 0 = slot-based activation

# Admin addresses
export BASE_KEY=$(solana-keygen pubkey $BASE_PATH)
export ADMIN=$(solana-keygen pubkey $KEYPAIR_PATH)
export CLAWBACK_RECEIVER_OWNER=$ADMIN  # Where unclaimed tokens go

# CLAIM TYPE CONFIGURATION
export CLAIM_TYPE=0  # Choose your claim type (see options below)

# OPTION A: Simple Community Airdrop (RECOMMENDED FOR YOUR USE CASE)
# CLAIM_TYPE=0 (Permissionless) - Eligible recipients can claim directly
export OPERATOR="11111111111111111111111111111111"  # Default pubkey (no operator needed)
export LOCKER="11111111111111111111111111111111"   # Default pubkey (no staking)

# OPTION B: Controlled Airdrop (Uncomment if you want additional admin approval)
# export CLAIM_TYPE=1  # Permissioned - requires merkle proof + admin signature
# export OPERATOR=$ADMIN  # Admin must co-sign each claim transaction
# export LOCKER="11111111111111111111111111111111"   # Default pubkey (no staking)

# OPTION C: Staking Integration (Advanced - requires staking infrastructure)
# export CLAIM_TYPE=2  # PermissionlessWithStaking
# export OPERATOR="11111111111111111111111111111111"  # Default pubkey
# export LOCKER="YourStakingLockerAddressHere"  # Your staking locker address

# Optional bonus parameters (set to 0 if not using)
export BONUS_VESTING_DURATION=0
export BONUS_MULTIPLIER=0
```

### 8. Generate Merkle Trees

```bash
# Create merkle tree proof files
# NOTE: The --amount parameter is misleading - it's only used for internal test lists.
# The actual token amounts come from your CSV file (recipients_30k.csv).
# You can set --amount to any value (e.g., 1000) as it won't affect your distribution.
./target/release/cli create-merkle-tree \
  --csv-path $CSV_PATH \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --max-nodes-per-tree 12000 \
  --amount 1000 \
  --decimals $TOKEN_DECIMALS

# Verify tree generation
ls -la $MERKLE_TREE_PATH/
```

## Phase 5: Distributor Deployment

### 9. Deploy Distributor

```bash
# Deploy the distributor
./target/release/cli \
  --mint $TOKEN_MINT \
  --priority-fee $PRIORITY_FEE \
  --keypair-path $KEYPAIR_PATH \
  --rpc-url $RPC \
  --program-id BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy \
  new-distributor \
  --start-vesting-ts $START_VESTING_TS \
  --end-vesting-ts $END_VESTING_TS \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --base-path $BASE_PATH \
  --clawback-start-ts $CLAWBACK_START_TS \
  --activation-point $ACTIVATION_POINT \
  --activation-type $ACTIVATION_TYPE \
  --clawback-receiver-owner $CLAWBACK_RECEIVER_OWNER \
  --closable \
  --bonus-vesting-duration $BONUS_VESTING_DURATION \
  --bonus-multiplier $BONUS_MULTIPLIER \
  --operator $OPERATOR \
  --locker $LOCKER \
  --claim-type $CLAIM_TYPE
```

### 10. Fund Distributor

```bash
# Fund all distributors with tokens
./target/release/cli \
  --mint $TOKEN_MINT \
  --priority-fee $PRIORITY_FEE \
  --base $BASE_KEY \
  --keypair-path $KEYPAIR_PATH \
  --rpc-url $RPC \
  --program-id BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy \
  fund-all \
  --merkle-tree-path $MERKLE_TREE_PATH
```

### 11. Verify Deployment

```bash
# Verify the distributor configuration
# Use the --rpc-url flag directly since verify command has issues with environment variables
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --program-id BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy \
  verify \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --clawback-start-ts $CLAWBACK_START_TS \
  --activation-point $ACTIVATION_POINT \
  --activation-type $ACTIVATION_TYPE \
  --admin $ADMIN \
  --clawback-receiver-owner $CLAWBACK_RECEIVER_OWNER \
  --closable \
  --bonus-multiplier $BONUS_MULTIPLIER \
  --claim-type $CLAIM_TYPE \
  --operator $OPERATOR \
  --locker $LOCKER

# Alternative: Check distributor status directly
./target/release/cli \
  --mint $TOKEN_MINT \
  --program-id BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy \
  --base $BASE_KEY \
  --rpc-url https://api.devnet.solana.com \
  view-distributors \
  --from-version 0 \
  --to-version 5
```

## Phase 6: API Server Setup

### 12. Start API Server

```bash
# Build API server
cargo build -p jupiter-airdrop-api --release

# Start the API server
./target/release/jupiter-airdrop-api \
  --bind-addr 0.0.0.0:7001 \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --base $BASE_KEY \
  --mint $TOKEN_MINT \
  --program-id BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy
```

### 13. Test API Endpoints

```bash
# Test distributor info
curl http://localhost:7001/distributors

# Test proof generation for a specific wallet
curl "http://localhost:7001/user/WALLET_ADDRESS_HERE"
```

## Phase 7: Frontend Integration

### 14. Key API Endpoints for Frontend

#### Get Distributor Information

```
GET /distributors
Response: List of all distributors with metadata
```

#### Get Claim Proof

```
GET /user/{WALLET_ADDRESS}
Response: Merkle proof required for claiming
```

#### Frontend Integration Example

**Important:** The on-chain program (`BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy`) has been updated. The `newClaim` instruction now requires the `operator` and `locker` accounts to be passed in, even for permissionless airdrops. For your configuration, these should be set to the system program's address (`11111111111111111111111111111111`).

```javascript
// Example claim function
async function claimTokens(walletAddress, connection, wallet) {
  // 1. Get proof from API
  const response = await fetch(`/user/${walletAddress}`);
  const proofData = await response.json();

  // 2. Build claim transaction (assuming you are using Anchor)
  const program = new Program(idl, "BZuXaMhhTG4cpHkgUHzz6pKhQrV4jdpZjmF5M3zi2HQy", provider);
  
  const transaction = await program.methods
    .newClaim(proofData.amount, proofData.locked_amount, proofData.proof)
    .accounts({
      distributor: new PublicKey(proofData.merkle_tree),
      // Ensure you derive the claim status PDA correctly
      claimStatus: /* PDA for claim status */,
      from: /* Distributor's token account (ATA) */,
      to: /* User's token account (ATA) */,
      claimant: wallet.publicKey,
      // NEW REQUIRED ACCOUNTS
      operator: new PublicKey("11111111111111111111111111111111"),
      locker: new PublicKey("11111111111111111111111111111111"),
      // System accounts
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .transaction();

  // 3. Sign and send
  const signature = await wallet.sendTransaction(transaction, connection);
  await connection.confirmTransaction(signature);
}
```

## Phase 8: Testing & Validation

### 15. Test Claims

```bash
# Generate proof for test wallet
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  kv-proof \
  --merkle-tree-path $MERKLE_TREE_PATH \
  --claimant TEST_WALLET_ADDRESS

# Test claim via API
curl -X POST http://localhost:7001/claim \
  -H "Content-Type: application/json" \
  -d '{"wallet":"TEST_WALLET_ADDRESS","proof":"..."}'
```

### 16. Monitor Distribution

```bash
# Check distributor account balances
solana account $DISTRIBUTOR_PUBKEY --url $RPC

# Monitor claims over time
./target/release/cli \
  --mint $TOKEN_MINT \
  --base $BASE_KEY \
  --rpc-url $RPC \
  stats \
  --merkle-tree-path $MERKLE_TREE_PATH
```

## Phase 9: Launch Preparation

### 17. Security Checklist

- [ ] Test claims with multiple wallet types
- [ ] Verify token account creation works correctly
- [ ] Confirm clawback functionality
- [ ] Test edge cases (double claims, invalid proofs)
- [ ] Load test API server with expected traffic
- [ ] Set up monitoring and logging

### 18. Go-Live Checklist

- [ ] API server running on production infrastructure
- [ ] Frontend integrated and tested
- [ ] Monitoring systems in place
- [ ] Support documentation ready
- [ ] Community announcements prepared

## Cost Breakdown (Devnet)

### Estimated Costs for 30K Recipients:

- **Program Deployment**: ~5 SOL (if deploying custom instance)
- **Distributor Creation**: ~1 SOL
- **Token Account Funding**: ~60 SOL (0.002 SOL × 30,000)
- **Priority Fees**: ~3-5 SOL (depending on network congestion)
- **Buffer**: ~10 SOL
- **Total**: ~80-85 SOL

## Troubleshooting

### Common Issues:

1. **Insufficient SOL**: Ensure admin wallet has enough SOL for all operations
2. **Invalid CSV**: Validate all wallet addresses and amounts
3. **RPC Rate Limits**: Use private RPC endpoints for large operations
4. **Memory Issues**: Split large CSV files if needed
5. **Transaction Failures**: Increase priority fees during high congestion

### Support Commands:

```bash
# Check distributor status
./target/release/cli status --mint $TOKEN_MINT --base $BASE_KEY --rpc-url $RPC

# Emergency clawback (if needed)
./target/release/cli clawback --mint $TOKEN_MINT --base $BASE_KEY --keypair-path $KEYPAIR_PATH --rpc-url $RPC

# Transfer admin (for production)
./target/release/cli set-admin --mint $TOKEN_MINT --base $BASE_KEY --keypair-path $KEYPAIR_PATH --rpc-url $RPC --new-admin MULTISIG_ADDRESS --merkle-tree-path $MERKLE_TREE_PATH
```

## Production Considerations

### 1. Security

- Transfer admin rights to a multisig wallet after deployment
- Use hardware wallets for key management
- Implement rate limiting on API endpoints

### 2. Infrastructure

- Use dedicated RPC endpoints (Helius, QuickNode, etc.)
- Set up load balancing for API servers
- Implement comprehensive logging and monitoring

### 3. User Experience

- Build intuitive claiming interface
- Provide clear instructions and support
- Consider integration with popular wallets

This guide provides a complete pathway from development setup to production deployment for your 30K recipient token distribution on Devnet.

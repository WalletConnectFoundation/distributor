extern crate jito_merkle_tree;
extern crate merkle_distributor;

pub mod instructions;
use std::{fs, path::PathBuf, rc::Rc, str::FromStr};

use anchor_client::{
    solana_sdk::signer::keypair::read_keypair_file, Client as AnchorClient, Cluster, Program,
};
use anchor_lang::{
    prelude::{Clock, Pubkey},
    solana_program::sysvar,
    AccountDeserialize, InstructionData, Key, ToAccountMetas,
};
use anchor_spl::token::{self, TokenAccount};
use anyhow::Result;
use bincode::deserialize;
use clap::{Parser, Subcommand};
use csv::Writer;
use jito_merkle_tree::{
    airdrop_merkle_tree::AirdropMerkleTree,
    csv_entry::CsvEntry,
    utils::{get_claim_status_pda, get_merkle_distributor_pda},
};
use merkle_distributor::state::merkle_distributor::MerkleDistributor;
use solana_program::{clock::DEFAULT_MS_PER_SLOT, instruction::Instruction};
use solana_rpc_client::rpc_client::{RpcClient, SerializableTransaction};
use solana_sdk::{
    account::Account,
    commitment_config::CommitmentConfig,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::instructions::*;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// SPL Mint address
    #[clap(long, env, default_value_t = Pubkey::default())]
    pub mint: Pubkey,

    /// base key for merkle tree
    #[clap(long, env, default_value_t = Pubkey::default())]
    pub base: Pubkey,

    /// RPC url
    #[clap(long, env, default_value = "http://localhost:8899")]
    pub rpc_url: String,

    /// Program id
    #[clap(long, env, default_value_t = merkle_distributor::id())]
    pub program_id: Pubkey,

    /// Payer keypair
    #[clap(long, env)]
    pub keypair_path: Option<String>,

    /// Priority fee
    #[clap(long, env)]
    pub priority_fee: Option<u64>,
}

impl Args {
    fn get_program_client(&self) -> Program<Rc<Keypair>> {
        let payer = Keypair::new();
        let client = AnchorClient::new_with_options(
            Cluster::Custom(self.rpc_url.clone(), self.rpc_url.clone()),
            Rc::new(Keypair::from_bytes(&payer.to_bytes()).unwrap()),
            CommitmentConfig::finalized(),
        );
        let program: anchor_client::Program<Rc<Keypair>> =
            client.program(merkle_distributor::id()).unwrap();
        program
    }
}

// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Claim unlocked tokens
    Claim(ClaimArgs),
    /// Claim by using jup API
    ClaimFromApi(ClaimFromApiArgs),
    /// Create a new instance of a merkle distributor
    NewDistributor(NewDistributorArgs),

    CloseDistributor(CloseDistributorArgs),
    CloseClaimStatus(CloseClaimStatusArgs),
    /// Clawback tokens from merkle distributor
    #[clap(hide = true)]
    Clawback(ClawbackArgs),
    /// Create a Merkle tree, given a CSV of recipients
    CreateMerkleTree(CreateMerkleTreeArgs),
    SetAdmin(SetAdminArgs),

    SetActivationPoint(SetActivationArgs),
    SetActivationSlotByTime(SetActivationSlotByTimeArgs),

    CreateTestList(CreateTestListArgs),
    CreateDummyCsv(CreateDummyCsv),
    ExtendList(ExtendListArgs),

    FundAll(FundAllArgs),
    Verify(VerifyArgs),
    FilterList(FilterListArgs),
    FilterListFixed(FilterListFixedArgs),
    FilterAndMergeList(FilterAndMergeListArgs),
    SlotByTime(SlotByTimeArgsArgs),
    /// generate kv proof
    GenerateKvProof(GenerateKvProofArgs),
    MassSend(MassSendArgs),
    Resend(ResendSendArgs),
    ViewClaimStatus(ViewClaimStatusArgs),

    VerifyKvProof(VerifyKvProofArgs),
    TotalClaim(TotalClaimAgrs),

    SetClawbackReceiver(ClawbackReceiverArgs),

    ViewDistributors(ViewDistributorsArgs),
    ViewDistributorByPubkey(ViewDistributorByPubkeyArgs),
    /// Upload merkle tree to PostgreSQL database
    UploadMerkleTree(UploadMerkleTreeArgs),
}

#[derive(Parser, Debug)]
pub struct CloseDistributorArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    pub airdrop_version: Option<u64>,
}

#[derive(Parser, Debug)]
pub struct CloseClaimStatusArgs {}
// NewClaim and Claim subcommand args
#[derive(Parser, Debug)]
pub struct ClaimArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ClaimFromApiArgs {
    /// Merkle distributor path
    #[clap(long, env, default_value = "https://worker.jup.ag/jup-claim-proof")]
    root_api: String,
    #[clap(long, env)]
    destination_owner: Pubkey,
}

#[derive(Parser, Debug)]
pub struct FundAllArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct VerifyArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// When to make the clawback period start. Must be at least a day after the end_vesting_ts
    #[clap(long, env)]
    pub clawback_start_ts: i64,

    #[clap(long, env)]
    pub clawback_receiver_owner: Pubkey,

    #[clap(long, env)]
    pub activation_point: u64,

    #[clap(long, env)]
    pub activation_type: u8,

    #[clap(long, env)]
    pub admin: Pubkey,

    #[clap(long, env)]
    pub airdrop_version: Option<u64>,

    #[clap(long, env)]
    pub closable: bool,

    #[clap(long, env)]
    pub skip_verify_amount: bool,

    // #[clap(long, env)]
    // pub bonus_vesting_duration: u64,
    #[clap(long, env)]
    pub bonus_multiplier: u64,

    #[clap(long, env)]
    pub claim_type: u8,

    #[clap(long, env)]
    pub operator: Pubkey,

    #[clap(long, env)]
    pub locker: Pubkey,
}

// NewDistributor subcommand args
#[derive(Parser, Debug)]
pub struct NewDistributorArgs {
    /// Lockup timestamp start
    #[clap(long, env)]
    pub start_vesting_ts: i64,

    /// Lockup timestamp end (unix timestamp)
    #[clap(long, env)]
    pub end_vesting_ts: i64,

    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// When to make the clawback period start. Must be at least a day after the end_vesting_ts
    #[clap(long, env)]
    pub clawback_start_ts: i64,

    #[clap(long, env)]
    pub activation_point: u64,

    #[clap(long, env)]
    pub activation_type: u8,

    #[clap(long, env)]
    pub airdrop_version: Option<u64>,

    #[clap(long, env)]
    pub closable: bool,

    #[clap(long, env)]
    pub skip_verify: bool,

    /// Base keypair
    #[clap(long, env)]
    pub base_path: String,

    /// Clawback receiver owner
    #[clap(long, env)]
    pub clawback_receiver_owner: Pubkey,

    #[clap(long, env)]
    pub bonus_vesting_duration: u64,

    #[clap(long, env)]
    pub bonus_multiplier: u64,

    #[clap(long, env)]
    pub claim_type: u8,

    #[clap(long, env)]
    pub operator: Pubkey,

    #[clap(long, env)]
    pub locker: Pubkey,
}

#[derive(Parser, Debug)]
pub struct ClawbackArgs {
    #[clap(long, env)]
    pub from_version: u64,
    #[clap(long, env)]
    pub to_version: u64,
}

#[derive(Parser, Debug)]
pub struct CreateMerkleTreeArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,

    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    /// max nodes per tree
    #[clap(long, env)]
    pub max_nodes_per_tree: u64,

    #[clap(long, env)]
    pub should_include_test_list: bool,

    #[clap(long, env)]
    pub amount: u64,
    #[clap(long, env)]
    pub decimals: u32,
}

#[derive(Parser, Debug)]
pub struct SetAdminArgs {
    #[clap(long, env)]
    pub new_admin: Pubkey,
    #[clap(long, env)]
    pub from_version: u64,
    #[clap(long, env)]
    pub to_version: u64,
}

#[derive(Parser, Debug)]
pub struct SetActivationArgs {
    #[clap(long, env)]
    pub from_version: u64,
    #[clap(long, env)]
    pub to_version: u64,
    #[clap(long, env)]
    pub activation_point: u64, // can be timestamp or slot
}

#[derive(Parser, Debug)]
pub struct SetActivationSlotByTimeArgs {
    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    pub timestamp: u64,
    #[clap(long, env)]
    pub airdrop_version: Option<u64>,
}

#[derive(Parser, Debug)]
pub struct SlotByTimeArgsArgs {
    #[clap(long, env)]
    pub timestamp: u64,
}

#[derive(Parser, Debug)]
pub struct CreateTestListArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,

    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,

    #[clap(long, env)]
    pub amount: u64,

    #[clap(long, env)]
    pub decimals: u32,
}

#[derive(Parser, Debug)]
pub struct CreateDummyCsv {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: String,
    #[clap(long, env)]
    pub num_records: u64,
    #[clap(long, env)]
    pub amount: u64,
}

#[derive(Parser, Debug)]
pub struct ExtendListArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,
    #[clap(long, env)]
    pub num_records: u64,
    #[clap(long, env)]
    pub amount: u64,
    #[clap(long, env)]
    pub destination_path: String,
}

#[derive(Parser, Debug)]
pub struct FilterListArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,
    #[clap(long, env)]
    pub amount: u64,
    #[clap(long, env)]
    pub destination_path: String,
}

#[derive(Parser, Debug)]
pub struct FilterListFixedArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,
    #[clap(long, env)]
    pub destination_path: String,
    #[clap(long, env)]
    pub decimals: u32,
}

#[derive(Parser, Debug)]
pub struct FilterAndMergeListArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,
    #[clap(long, env)]
    pub sub_path: PathBuf,
    #[clap(long, env)]
    pub amount: u64,
    #[clap(long, env)]
    pub destination_path: String,
}

#[derive(Parser, Debug)]
pub struct GenerateKvProofArgs {
    /// Merkle tree out path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    pub kv_path: PathBuf,
    #[clap(long, env)]
    pub max_entries_per_file: u64,
}

#[derive(Parser, Debug)]
pub struct MassSendArgs {
    /// CSV path
    #[clap(long, env)]
    pub csv_path: PathBuf,
    /// CSV path
    #[clap(long, env)]
    pub des_path: PathBuf,
    #[clap(long, env)]
    pub max_address_per_tx: u64,
    #[clap(long, env)]
    pub amount: u64,
}

#[derive(Parser, Debug)]
pub struct ResendSendArgs {
    #[clap(long, env)]
    pub des_path: PathBuf,
    #[clap(long, env)]
    pub max_address_per_tx: u64,
    #[clap(long, env)]
    pub amount: u64,
}

#[derive(Parser, Debug)]
pub struct ViewClaimStatusArgs {}

#[derive(Parser, Debug)]
pub struct VerifyKvProofArgs {
    /// list of all user and allocation
    #[clap(long, env)]
    pub csv_path: PathBuf,
    /// kv endpoint
    #[clap(long, env)]
    pub kv_api: String,
    /// local api
    #[clap(long, env)]
    pub local_api: String,
    /// number of entries to verify
    #[clap(long, env)]
    pub num_verify: u64,
}

#[derive(Parser, Debug)]
pub struct TotalClaimAgrs {
    #[clap(long, env)]
    pub num_tree: u64,
}

#[derive(Parser, Debug)]
pub struct ClawbackReceiverArgs {
    /// Merkle distributor path
    #[clap(long, env)]
    pub merkle_tree_path: PathBuf,
    #[clap(long, env)]
    receiver: Pubkey,
}

#[derive(Parser, Debug)]
pub struct ViewDistributorsArgs {
    #[clap(long, env)]
    pub from_version: u64,
    #[clap(long, env)]
    pub to_version: u64,
}

#[derive(Parser, Debug)]
pub struct ViewDistributorByPubkeyArgs {
    #[clap(long, env)]
    pub pubkey: Pubkey,
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Commands::NewDistributor(new_distributor_args) => {
            process_new_distributor(&args, new_distributor_args);
        }
        Commands::CloseDistributor(close_distributor_args) => {
            process_close_distributor(&args, close_distributor_args);
        }
        Commands::Claim(claim_args) => {
            process_claim(&args, claim_args);
        }
        Commands::ClaimFromApi(claim_args) => {
            process_claim_from_api(&args, claim_args);
        }
        Commands::Clawback(clawback_args) => process_clawback(&args, clawback_args),
        Commands::CreateMerkleTree(merkle_tree_args) => {
            process_create_merkle_tree(merkle_tree_args);
        }
        Commands::SetAdmin(set_admin_args) => {
            process_set_admin(&args, set_admin_args);
        }
        Commands::SetActivationPoint(sub_args) => {
            process_set_activation_point(&args, sub_args);
        }
        Commands::SetActivationSlotByTime(sub_args) => {
            process_set_activation_slot_by_time(&args, sub_args);
        }
        Commands::CreateDummyCsv(test_args) => {
            process_create_dummy_csv(test_args);
        }
        Commands::CreateTestList(create_test_list_args) => {
            process_create_test_list(&args, create_test_list_args);
        }
        Commands::FundAll(fund_all_args) => {
            process_fund_all(&args, fund_all_args);
        }
        Commands::Verify(verfiy_args) => {
            process_verify(&args, verfiy_args);
        }
        Commands::ExtendList(extend_list_args) => {
            process_extend_list(extend_list_args);
        }
        Commands::FilterList(filter_list_args) => {
            process_filter_list(filter_list_args);
        }
        Commands::FilterListFixed(filter_list_args) => {
            process_filter_list_fixed(filter_list_args);
        }
        Commands::SlotByTime(slot_by_time_args) => {
            process_get_slot(&args, slot_by_time_args);
        }
        Commands::CloseClaimStatus(_args) => {
            process_close_claim_status(&args);
        }
        Commands::FilterAndMergeList(filter_and_merge_list_args) => {
            process_filter_and_merge(filter_and_merge_list_args);
        }
        Commands::GenerateKvProof(generate_kv_proof_args) => {
            process_generate_kv_proof(&args, generate_kv_proof_args);
        }
        Commands::MassSend(mass_send_args) => process_mass_send(&args, mass_send_args),
        Commands::Resend(re_send_args) => process_resend(&args, re_send_args),
        Commands::ViewClaimStatus(_view_claim_status_args) => view_claim_status(&args),
        Commands::VerifyKvProof(verify_kv_proof_args) => verify_kv_proof(verify_kv_proof_args),
        Commands::TotalClaim(total_claim_argrs) => get_total_claim(&args, total_claim_argrs),
        Commands::ViewDistributors(view_distributors_args) => {
            view_distributors(&args, view_distributors_args)
        }
        Commands::ViewDistributorByPubkey(sub_args) => {
            view_distributor_by_pubkey(&args, &sub_args.pubkey)
        }
        Commands::SetClawbackReceiver(set_clawback_receiver_argrs) => {
            process_set_clawback_receiver(&args, set_clawback_receiver_argrs)
        }
        Commands::UploadMerkleTree(upload_args) => {
            // Create a new args struct with the required parameters from the main Args
            let mut enhanced_upload_args = upload_args.clone();
            enhanced_upload_args.program_id = args.program_id;
            enhanced_upload_args.base = args.base;
            enhanced_upload_args.mint = args.mint;
            
            if let Err(e) = process_upload_merkle_tree_sync(&enhanced_upload_args) {
                eprintln!("Error uploading merkle tree: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn check_distributor_onchain_matches(
    account: &Account,
    merkle_tree: &AirdropMerkleTree,
    new_distributor_args: &NewDistributorArgs,
    total_bonus: u64,
    pubkey: Pubkey,
    base: Pubkey,
    args: &Args,
) -> Result<(), &'static str> {
    if let Ok(distributor) = MerkleDistributor::try_deserialize(&mut account.data.as_slice()) {
        if distributor.root != merkle_tree.merkle_root {
            return Err("root mismatch");
        }

        if distributor.base != base {
            return Err("base mismatch");
        }

        if distributor.max_total_claim
            != merkle_tree
                .get_max_total_claim()
                .checked_add(total_bonus)
                .unwrap()
        {
            return Err("max_total_claim mismatch");
        }
        if distributor.max_num_nodes != merkle_tree.max_num_nodes {
            return Err("max_num_nodes mismatch");
        }

        if distributor.start_ts != new_distributor_args.start_vesting_ts {
            return Err("start_ts mismatch");
        }
        if distributor.end_ts != new_distributor_args.end_vesting_ts {
            return Err("end_ts mismatch");
        }
        if distributor.clawback_start_ts != new_distributor_args.clawback_start_ts {
            return Err("clawback_start_ts mismatch");
        }

        if distributor.activation_type != new_distributor_args.activation_type {
            return Err("activation_type mismatch");
        }
        if distributor.activation_point != new_distributor_args.activation_point {
            return Err("activation_slot mismatch");
        }

        if distributor.closable() != new_distributor_args.closable {
            return Err("closable mismatch");
        }

        if distributor.airdrop_bonus.total_bonus != total_bonus {
            return Err("total_bonus mismatch");
        }

        if distributor.airdrop_bonus.vesting_duration != new_distributor_args.bonus_vesting_duration
        {
            return Err("bonus_vesting_duration mismatch");
        }

        if distributor.operator != new_distributor_args.operator {
            return Err("operator mismatch");
        }

        if distributor.locker != new_distributor_args.locker {
            return Err("locker mismatch");
        }

        // TODO fix code
        let clawback_receiver_token_account =
            spl_associated_token_account::get_associated_token_address(
                &new_distributor_args.clawback_receiver_owner,
                &args.mint,
            );

        if clawback_receiver_token_account != distributor.clawback_receiver {
            return Err("clawback_receiver mismatch");
        }
        if distributor.admin != pubkey {
            return Err("admin mismatch");
        }
    }
    Ok(())
}

fn get_pre_list() -> Vec<String> {
    let list = vec![
        "DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX",
        "BULRqL3U2jPgwvz6HYCyBVq9BMtK94Y1Nz98KQop23aD",
        "7w32LzRsJrQiE7S3ZSdkz9TSFGey1XNsonPmdm9xDUch",
        "55pPhcCcp8gEKvKWr1JUkAcdwMeemmNhTHmkWNR9sJib",
        "62ucxc2gd5TBCwzToEEWVV4M5drVK7Fi7aYozniqWtac",
        "5unTfT2kssBuNvHPY6LbJfJpLqEcdMxGYLWHwShaeTLi",
        "9zg3seAh4Er1Nz8GAuiciH437apxtzgUWBT8frhudevR",
        "AjefJWRfjRCVNSQ1pHnTW8F7szLV7xFZftiB3yM5vnTa",
        "8SEFruHjgNrnV8ak2Ff11wg9em8Nh72RWTwk359bRyzE",
        "7jBypy9HX1dyLHPnmRnRubibNUaBPrShnERGnoE7rc3C",
        "XWpxVfYTeKmmp18DPxqPvWFL7P1C2vbdegDPAbXkV1n",
        "AuTFdqo4GsxpDgtag87pDaHE259cE94Z82kdpFozVBhC",
        "6h43GsVT3TjtLa5nRpsXp15GDpAY4smWCYHgcq58dSPM",
        "2mAax9cNqDXDg9eDJDby1tBh9Q8N3TS7qLhX9rMp8EVc",
        "JBeYA7dmBGCNgaEdtqdoUnESwKJho5YvgXVNLgo4n3MM",
        "HeTpE5BnNinzNv92MzVAGyVT5LjAwTWuk5qQcPURmi2L",
        "Bidku3jkJUxiTzBJZroEfwPcUWueNUst9LrMbZQLhrtG",
        "HUQytvb7WCCqbHnpQrVgXhmXSw4XfWMnmqCiKz6T1vsU",
        "4zvTjdpyr3SAgLeSpCnq4KaHvX2j5SbkwxYydzbfqhRQ",
        "EVfUfs9XNwJmfNvoazDbZVb6ecnGCxgQrJzsCQHoQ4q7",
        "GMtwcuktJfrRcnyGktWW4Vab8cfjPcBy3xbuZgRegw6E",
        "HAPdsaZFfQDG4bD8vzBbPCUawUWKSJxvhQ7TGg1BeAxZ",
    ];
    let list: Vec<String> = list.iter().map(|x| x.to_string()).collect();
    list
}

fn get_test_list() -> Vec<String> {
    let list = vec![
        "62ucxc2gd5TBCwzToEEWVV4M5drVK7Fi7aYozniqWtac",
        "HcrGezZ4vsveZ9vRX1vQU3WRiEzx8T4XcBBv9Hs9Smmd",
        "DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX",
    ];
    let list: Vec<String> = list.iter().map(|x| x.to_string()).collect();
    list
}

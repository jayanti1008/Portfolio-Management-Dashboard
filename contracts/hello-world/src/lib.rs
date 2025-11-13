#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to track individual asset holdings
#[contracttype]
#[derive(Clone)]
pub struct AssetHolding {
    pub asset_name: String,
    pub asset_code: String,
    pub quantity: i64,
    pub purchase_price: i64,
    pub current_price: i64,
    pub timestamp: u64,
}

// Structure to track portfolio summary
#[contracttype]
#[derive(Clone)]
pub struct PortfolioSummary {
    pub total_assets: u64,
    pub total_value: i64,
    pub last_updated: u64,
}

// Mapping user address to their portfolio summary
#[contracttype]
pub enum PortfolioBook {
    Summary(Address),
    Asset(Address, u64), // (user_address, asset_id)
}

// Counter for asset IDs per user
const ASSET_COUNTER: Symbol = symbol_short!("ASSET_CT");

#[contract]
pub struct PortfolioContract;

#[contractimpl]
impl PortfolioContract {
    
    // Function to add a new asset to user's portfolio
    pub fn add_asset(
        env: Env,
        user: Address,
        asset_name: String,
        asset_code: String,
        quantity: i64,
        purchase_price: i64,
    ) -> u64 {
        user.require_auth();
        
        // Get or initialize asset counter for this user
        let counter_key = (ASSET_COUNTER, user.clone());
        let mut asset_count: u64 = env.storage().persistent().get(&counter_key).unwrap_or(0);
        asset_count += 1;
        
        let timestamp = env.ledger().timestamp();
        
        // Create new asset holding
        let new_asset = AssetHolding {
            asset_name: asset_name.clone(),
            asset_code: asset_code.clone(),
            quantity,
            purchase_price,
            current_price: purchase_price, // Initially same as purchase price
            timestamp,
        };
        
        // Store the asset
        let asset_key = PortfolioBook::Asset(user.clone(), asset_count);
        env.storage().persistent().set(&asset_key, &new_asset);
        
        // Update portfolio summary
        let mut summary = Self::get_portfolio_summary(env.clone(), user.clone());
        summary.total_assets += 1;
        summary.total_value += quantity * purchase_price;
        summary.last_updated = timestamp;
        
        let summary_key = PortfolioBook::Summary(user.clone());
        env.storage().persistent().set(&summary_key, &summary);
        
        // Update counter
        env.storage().persistent().set(&counter_key, &asset_count);
        
        env.storage().persistent().extend_ttl(&asset_key, 5000, 5000);
        env.storage().persistent().extend_ttl(&summary_key, 5000, 5000);
        
        log!(&env, "Asset added successfully with ID: {}", asset_count);
        asset_count
    }
    
    // Function to update the current price of an asset
    pub fn update_asset_price(
        env: Env,
        user: Address,
        asset_id: u64,
        new_price: i64,
    ) {
        user.require_auth();
        
        let asset_key = PortfolioBook::Asset(user.clone(), asset_id);
        let mut asset: AssetHolding = env.storage().persistent()
            .get(&asset_key)
            .unwrap_or_else(|| panic!("Asset not found"));
        
        let old_value = asset.quantity * asset.current_price;
        asset.current_price = new_price;
        let new_value = asset.quantity * new_price;
        
        env.storage().persistent().set(&asset_key, &asset);
        
        // Update portfolio summary
        let summary_key = PortfolioBook::Summary(user.clone());
        let mut summary: PortfolioSummary = env.storage().persistent()
            .get(&summary_key)
            .unwrap_or_else(|| panic!("Portfolio not found"));
        
        summary.total_value = summary.total_value - old_value + new_value;
        summary.last_updated = env.ledger().timestamp();
        
        env.storage().persistent().set(&summary_key, &summary);
        
        env.storage().persistent().extend_ttl(&asset_key, 5000, 5000);
        env.storage().persistent().extend_ttl(&summary_key, 5000, 5000);
        
        log!(&env, "Asset price updated for ID: {}", asset_id);
    }
    
    // Function to get portfolio summary for a user
    pub fn get_portfolio_summary(env: Env, user: Address) -> PortfolioSummary {
        let summary_key = PortfolioBook::Summary(user);
        
        env.storage().persistent().get(&summary_key).unwrap_or(PortfolioSummary {
            total_assets: 0,
            total_value: 0,
            last_updated: 0,
        })
    }
    
    // Function to view specific asset details
    pub fn view_asset(env: Env, user: Address, asset_id: u64) -> AssetHolding {
        let asset_key = PortfolioBook::Asset(user, asset_id);
        
        env.storage().persistent().get(&asset_key).unwrap_or(AssetHolding {
            asset_name: String::from_str(&env, "Not_Found"),
            asset_code: String::from_str(&env, "N/A"),
            quantity: 0,
            purchase_price: 0,
            current_price: 0,
            timestamp: 0,
        })
    }
}
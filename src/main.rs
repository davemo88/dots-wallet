use std::sync::Arc;
use std::convert::Infallible;
use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use lru::LruCache;
use warp::Filter;

mod db;
mod error;
mod handlers;

use db::DB;
use handlers::{
    add_item_handler,
    retrieve_item_handler,
    create_wallet_handler,
};

pub type WalletId = u32;
pub type ItemId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: WalletId,
    pub items: Vec<ItemId>,
}

impl Wallet {
    pub fn new(id: WalletId) -> Self {
        Self {
            id,
            items: vec![]
        }
    }
}

// toy cache size
const CACHE_SIZE: usize = 10;
pub type WalletCache = Arc<RwLock<LruCache<WalletId, HashSet<ItemId>>>>;

#[tokio::main]
async fn main() {

// cache recently accessed wallets
    let wallet_cache: WalletCache = Arc::new(RwLock::new(LruCache::new(CACHE_SIZE)));

// mongodb client
    let wallet_db = DB::new().await.unwrap();
    wallet_db.init().await.unwrap();

// endpoints
    let create_wallet = warp::path("wallet")
        .and(warp::filters::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_shared(wallet_cache.clone()))
        .and(with_shared(wallet_db.clone()))
        .and_then(create_wallet_handler);

    let add_item = warp::path("wallet")
        .and(warp::path::param::<WalletId>())
        .and(warp::filters::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_shared(wallet_cache.clone()))
        .and(with_shared(wallet_db.clone()))
        .and_then(add_item_handler);

    let retrieve_item = warp::path("wallet")
        .and(warp::path::param::<WalletId>())
        .and(warp::path("item"))
        .and(warp::path::param::<ItemId>())
        .and(warp::filters::path::end())
        .and(warp::get())
        .and(with_shared(wallet_cache.clone()))
        .and(with_shared(wallet_db.clone()))
        .and_then(retrieve_item_handler);

    let routes = create_wallet
        .or(add_item)
        .or(retrieve_item)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0,0,0,0], 5000)).await;
}

// wraps shared resources for use by handler functions
fn with_shared<T: Clone + Send>(shared_resource: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || shared_resource.clone())
}

#[cfg(test)]
mod tests {
}

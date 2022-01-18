use tokio::sync::RwLock;
use std::sync::Arc;
use std::convert::Infallible;
use lru::LruCache;
use std::collections::HashSet;
use warp::Filter;

mod db;
mod error;
mod handlers;

use db::{
    DB,
    WalletId,
    ItemId,
};
use handlers::{
    add_item_handler,
    retrieve_item_handler,
    create_wallet_handler,
};

const CACHE_SIZE: usize = 2;
pub type WalletCache = Arc<RwLock<LruCache<WalletId, HashSet<ItemId>>>>;

fn with_shared<T: Clone + Send>(shared_resource: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || shared_resource.clone())
}

#[tokio::main]
async fn main() {

    let wallet_cache: WalletCache = Arc::new(RwLock::new(LruCache::new(CACHE_SIZE)));

    let wallet_db = DB::new().await.unwrap();
    wallet_db.init().await.unwrap();

    let create_wallet = warp::path("wallet")
        .and(warp::filters::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_shared(wallet_cache.clone()))
        .and(with_shared(wallet_db.clone()))
        .and_then(create_wallet_handler);

    let add_item = warp::path("wallet")
        .and(warp::path::param::<u32>())
        .and(warp::filters::path::end())
        .and(warp::post())
        .and(warp::body::json())
        .and(with_shared(wallet_cache.clone()))
        .and(with_shared(wallet_db.clone()))
        .and_then(add_item_handler);

    let retrieve_item = warp::path("wallet")
        .and(warp::path::param::<u32>())
        .and(warp::path("item"))
        .and(warp::path::param::<u32>())
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

#[cfg(test)]
mod tests {
}

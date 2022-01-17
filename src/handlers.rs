use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use warp::{ Reply, Rejection };
use crate::{
    DB,
    db::Wallet,
    WalletId,
    ItemId,
    WalletCache,
};

type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonResponse<T: Serialize> {
    pub status: Status,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T: Serialize> JsonResponse<T> {
    fn success(data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Success,
            data,
            message: None,
        }
    }

    fn error(message: String, data: Option<T>) -> Self {
        JsonResponse {
            status: Status::Error,
            data,
            message: Some(message),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CreateWalletBody {
    V1Body(WalletId)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AddItemBody {
    V1Body(ItemId)
}

pub async fn create_wallet_handler(body: CreateWalletBody, wallet_cache: WalletCache, wallet_db: DB) -> WebResult<impl Reply> {
    let wallet_id = match body {
        CreateWalletBody::V1Body(id) => id
    };
    let wallet_exists = {
        let wallets = wallet_cache.read().await;    
        wallets.contains(&wallet_id) || 
            wallet_db.get_wallet(wallet_id).await.is_ok()
    };
    if wallet_exists {
        let response = JsonResponse::<String>::error(format!(
                "wallet already exists for id {}", wallet_id), None);
        return Ok(warp::reply::json(&response));
    }
    match wallet_db.insert_wallet(wallet_id).await {
        Ok(()) => {
            put_wallet_in_cache(Wallet::new(wallet_id), wallet_cache.clone()).await;
            Ok(warp::reply::json(&
                JsonResponse::success(Some(wallet_id))))
        }
        Err(e) => Ok(warp::reply::json(&
                JsonResponse::<String>::error(e.to_string(), None)))
    }
}

pub async fn add_item_handler(wallet_id: u32, body: AddItemBody, wallet_cache: WalletCache, wallet_db: DB) -> WebResult<impl Reply> {
    let item_id = match body {
        AddItemBody::V1Body(item_id) => item_id
    };
    match wallet_db.add_item(wallet_id, item_id).await {
        Ok(()) => {
            Ok(warp::reply::json(&JsonResponse::success(Some((wallet_id, item_id))))),
// TODO: if wallet in cache, just update the item set
// else fetch the wallet from the db
// slightly inefficient because we an extra query to get the wallet twice to update the value
        }
        Err(e) => Ok(warp::reply::json(&
                JsonResponse::<String>::error(e.to_string(), None)))
    }
}

pub async fn retrieve_item_handler(wallet_id: u32, item_id: u32, wallet_cache: WalletCache, wallet_db: DB) -> WebResult<impl Reply> {
// check the cache, if not there fetch the wallet and then put it in the cache
    Ok(warp::reply::json(&JsonResponse::success(Some("ok".to_string()))))
}

async fn put_wallet_in_cache(wallet: Wallet, wallet_cache: WalletCache) {
    let mut wallet_cache = wallet_cache.write().await;
    let item_set = wallet.items.iter().copied().collect::<HashSet<ItemId>>();
    let _ = wallet_cache.put(wallet.id, item_set);
}

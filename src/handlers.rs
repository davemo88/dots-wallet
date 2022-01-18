use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use warp::{ Reply, Rejection };
use crate::{
    DB,
    db::Wallet,
    error::Error,
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
        let wallet_cache = wallet_cache.read().await;    
        wallet_cache.contains(&wallet_id) || 
            wallet_db.get_wallet(wallet_id).await.is_ok()
    };
    if wallet_exists {
        return Ok(warp::reply::json(&JsonResponse::<String>::error(
            Error::WalletAlreadyExists.to_string(), None)));
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
        Ok(wallet) => {
            put_wallet_in_cache(wallet, wallet_cache).await;
            Ok(warp::reply::json(&JsonResponse::success(Some((wallet_id, item_id)))))
        }
        Err(e) => Ok(warp::reply::json(&
                JsonResponse::<String>::error(e.to_string(), None)))
    }
}

pub async fn retrieve_item_handler(wallet_id: u32, item_id: u32, wallet_cache: WalletCache, wallet_db: DB) -> WebResult<impl Reply> {
// check the cache, if not there fetch the wallet and then put it in the cache
    let wallet_items = {
        let mut wallet_cache = wallet_cache.write().await;
        wallet_cache.get(&wallet_id).map(|items| items.clone())
    };
    let wallet_items = match wallet_items {
        Some(wallet_items) => wallet_items,
// if not in cache, fetch from db
        None => {
            let wallet = wallet_db.get_wallet(wallet_id).await;
            match wallet {
                Ok(wallet) => {
                    put_wallet_in_cache(wallet.clone(), wallet_cache).await;
                    wallet.items.iter().copied().collect()
                }
                Err(e)=> return Ok(warp::reply::json(&JsonResponse::<String>::error(
                    e.to_string(), None)))
            }
        }
    };
    if wallet_items.contains(&item_id) {
        Ok(warp::reply::json(&JsonResponse::success(Some(item_id))))
    } else {
        Ok(warp::reply::json(&JsonResponse::<String>::error(
                    Error::NoSuchItem.to_string(), None)))
    }
}

async fn put_wallet_in_cache(wallet: Wallet, wallet_cache: WalletCache) {
    let item_set = wallet.items.iter().copied().collect::<HashSet<ItemId>>();
    let mut wallet_cache = wallet_cache.write().await;
    let _ = wallet_cache.put(wallet.id, item_set);
}

#[cfg(test)]
mod test {

    #[test]
    fn test_print_request_body_json() {
        use super::*;
        let create_wallet_body = CreateWalletBody::V1Body(10);
        let add_item_body = AddItemBody::V1Body(10);
        println!("{}", serde_json::to_string(&create_wallet_body).unwrap());
        println!("{}", serde_json::to_string(&add_item_body).unwrap());
    }
}

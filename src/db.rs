use std::env;
use mongodb::{
    Client, 
    Collection,
    bson::doc,
    IndexModel,
    options::{ClientOptions, IndexOptions}, 
};
use crate::error::Error;
use crate::{
    WalletId,
    ItemId,
    Wallet
};

type Result<T> = std::result::Result<T, Error>;

const APP_NAME: &'static str = "dots-wallet";
const DB_NAME: &'static str = "wallets";
const COLLECTION_NAME: &'static str = "wallets";

#[derive(Clone, Debug)]
pub struct DB {
    client: Client,
}

impl DB {
    pub async fn new() -> Result<Self> {
        let user = env::var("MONGO_USERNAME").unwrap_or("root".into());
        let pass = env::var("MONGO_PASSWORD").unwrap_or("secret".into());
        let host = env::var("MONGO_HOST").unwrap_or("localhost".into());
        let port = env::var("MONGO_PORT").unwrap_or("27017".into());
        let mut options = ClientOptions::parse(format!("mongodb://{}:{}@{}:{}", user, pass, host, port)).await?;
        options.app_name = Some(APP_NAME.into());
        options.default_database = Some(DB_NAME.into());

        Ok(Self { 
            client: Client::with_options(options)? 
        })
    }

    pub async fn init(&self) -> Result<()> {
        let db = self.client.default_database().unwrap();

// add a wallet collection if it doesn't exist
        if !db.list_collection_names(None).await?.contains(&COLLECTION_NAME.into()) {
            db.create_collection(COLLECTION_NAME, None).await?;
            let collection = db.collection::<Wallet>("wallets");
            let options = IndexOptions::builder()
                .unique(true)
                .build();
            let index = IndexModel::builder()
                .keys(doc!{"id":1})
                .options(options)
                .build();
            collection.create_index(index, None).await?;
        }

        Ok(())
    }

    fn wallet_collection(&self) -> Collection<Wallet> {
        let db = self.client.default_database().unwrap();
        db.collection::<Wallet>(COLLECTION_NAME)
    }

    pub async fn get_wallet(&self, wallet_id: WalletId) -> Result<Wallet> {
        let filter = doc!{"id": wallet_id};
        match self.wallet_collection().find_one(filter, None).await? {
            Some(wallet) => Ok(wallet),
            None => Err(Error::NoSuchWallet),
        }
    }

    pub async fn insert_wallet(&self, wallet_id: WalletId) -> Result<()> {
        let _inserted_id = self.wallet_collection()
            .insert_one(Wallet::new(wallet_id), None).await?;
        Ok(())
    }

    pub async fn add_item(&self, wallet_id: WalletId, item_id: ItemId) -> Result<Wallet> {
        let mut wallet = self.get_wallet(wallet_id).await?;
        if wallet.items.contains(&item_id) {
            return Err(Error::ItemAlreadyInWallet)
        }
        wallet.items.push(item_id);
        self.wallet_collection().update_one(
            doc!{"id": wallet_id},
            doc!{"$set": {"items": wallet.items.clone()}},
            None
        ).await?;
        Ok(wallet)
    }
}

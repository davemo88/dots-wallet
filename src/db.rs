use serde::{Serialize, Deserialize};
use mongodb::{
    Client, 
    Collection,
    bson::doc,
    IndexModel,
    options::{ClientOptions, IndexOptions}, 
};
//use anyhow::Result;
use crate::error::Error;
type Result<T> = std::result::Result<T, Error>;

pub type WalletId = u32;
pub type ItemId = u32;

const MONGO_HOST: &'static str = "localhost";
const MONGO_PORT: &'static str = "27017";
const MONGO_USERNAME: &'static str = "root";
const MONGO_PASSWORD: &'static str = "secret";
const APP_NAME: &'static str = "dots-wallet";
const DB_NAME: &'static str = "wallets";
const COLLECTION_NAME: &'static str = "wallets";

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

#[derive(Clone, Debug)]
pub struct DB {
    client: Client,
}

impl DB {
    pub async fn new() -> Result<Self> {
        let mut options = ClientOptions::parse(format!("mongodb://{}:{}@{}:{}", MONGO_USERNAME, MONGO_PASSWORD, MONGO_HOST, MONGO_PORT)).await?;
        options.app_name = Some(APP_NAME.into());
        options.default_database = Some(DB_NAME.into());

        Ok(Self { 
            client: Client::with_options(options)? 
        })
    }

    pub async fn init(&self) -> Result<()> {
        let db = self.client.default_database().unwrap();

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
        let collection = self.wallet_collection();
        let filter = doc! { "id": wallet_id };
        match collection.find_one(filter, None).await? {
            Some(wallet) => Ok(wallet),
            None => Err(Error::NoSuchWallet),
        }
    }

    pub async fn insert_wallet(&self, wallet_id: WalletId) -> Result<()> {
        let new_wallet = Wallet::new(wallet_id); 
        let collection = self.wallet_collection();
        let _inserted_id = collection.insert_one(new_wallet,None).await?;
        Ok(())
    }

    pub async fn add_item(&self, wallet_id: WalletId, item_id: ItemId) -> Result<Wallet> {
        let mut wallet = self.get_wallet(wallet_id).await?;
        if wallet.items.contains(&item_id) {
            return Err(Error::ItemAlreadyInWallet)
        }
        wallet.items.push(item_id);
        let collection = self.wallet_collection();
        collection.update_one(
            doc! {"id": wallet_id },
            doc! {"$set": {"items": wallet.clone().items }},
            None
        ).await?;
        Ok(wallet)
    }
}

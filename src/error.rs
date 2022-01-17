use mongodb::error::Error as MongoError;
#[derive(Debug)]
pub enum Error {
    WalletAlreadyExists,
    NoSuchWallet,
    NoSuchItem,
    ItemAlreadyInWallet,
    Db(MongoError),
    
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::WalletAlreadyExists => write!(f, "wallet already exists"),
            Error::NoSuchWallet => write!(f, "no such wallet"),
            Error::NoSuchItem => write!(f, "no such item"),
            Error::ItemAlreadyInWallet => write!(f, "item already in wallet"),
            Error::Db(err) => write!(f, "Db({})", format!("{}", err.to_string())),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Db(e) => Some(e),
            _ => None,
        }
    }
}

impl From<MongoError> for Error {
    fn from(error: MongoError) -> Self {
        Error::Db(error.into())
    }
}

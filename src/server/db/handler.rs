use std::pin::Pin;

use tokio::sync::RwLock;

use anyhow::Result;
use futures_util::Future;

type AsyncCallback = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

pub struct WriteRequest {
    pub table: String,
    pub key: String,
    pub value: Vec<u8>,
    pub callback: Option<AsyncCallback>,
}

pub struct DBHandler {
    pub db: sled::Db,
    write_queue: RwLock<Vec<WriteRequest>>,
}

impl DBHandler {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self {
            db,
            write_queue: RwLock::new(vec![]),
        })
    }
}

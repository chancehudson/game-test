use std::collections::HashMap;
use std::pin::Pin;

use anyhow::Result;
use futures_util::Future;
use redb::Database;
use redb::TableDefinition;

use super::PLAYER_TABLE;

type AsyncCallback = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

pub struct WriteRequest {
    pub table: String,
    pub key: String,
    pub value: Vec<u8>,
    pub callback: Option<AsyncCallback>,
}

pub struct DBHandler {
    pub db: Database,
    table_map: HashMap<String, TableDefinition<'static, String, Vec<u8>>>,
    write_queue: Vec<WriteRequest>,
}

impl DBHandler {
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Database::create(db_path)?;
        let mut table_map = HashMap::new();
        table_map.insert("players".to_string(), PLAYER_TABLE);
        {
            let write = db.begin_write()?;
            write.open_table(PLAYER_TABLE)?;
            write.commit()?;
        }
        Ok(Self {
            table_map,
            db,
            write_queue: vec![],
        })
    }

    pub fn write(&mut self, req: WriteRequest) {
        self.write_queue.push(req);
    }

    pub async fn commit(&mut self) -> Result<()> {
        if self.write_queue.is_empty() {
            return Ok(());
        }
        let write_tx = self.db.begin_write()?;
        let mut callbacks = vec![];
        for req in self.write_queue.drain(..) {
            let table = self.table_map.get(&req.table);
            if table.is_none() {
                println!("attempt to write to non-existent table {}", req.table);
                continue;
            }
            let table = table.unwrap();
            let mut table = write_tx.open_table(*table)?;
            table.insert(req.key, req.value)?;
            if let Some(callback) = req.callback {
                callbacks.push(callback);
            }
        }
        write_tx.commit()?;
        for cb in callbacks {
            cb.await;
        }
        Ok(())
    }
}

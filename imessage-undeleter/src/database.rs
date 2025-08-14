/*!
Real iMessage Database Parser
Connects to the actual iMessage SQLite database and reads real message data.
*/

use rusqlite::{Connection, Result as SqlResult, Row};
use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealMessage {
    pub id: i32,
    pub guid: String,
    pub text: Option<String>,
    pub handle_id: Option<i32>,
    pub date: i64,
    pub date_edited: Option<i64>,
    pub date_retracted: Option<i64>,
    pub is_from_me: bool,
    pub cache_has_attachments: bool,
}

#[derive(Debug, Clone)]
pub struct Handle {
    pub id: i32, 
    pub identifier: String, 
    pub service: String, 
}

pub struct IMessageDatabase {
    conn: Connection,
    handle_cache: HashMap<i32, Handle>,
}

impl IMessageDatabase {
    pub fn new<P: AsRef<Path>>(db_path: P) -> SqlResult<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        
        let mut db = Self {
            conn,
            handle_cache: HashMap::new(),
        };
        
        db.load_handles()?;
        
        info!("Connected to iMessage database, loaded {} handles", db.handle_cache.len());
        Ok(db)
    }
    
    fn load_handles(&mut self) -> SqlResult<()> {
        let mut stmt = self.conn.prepare(
            "SELECT ROWID, id, service FROM handle"
        )?;
        
        let handle_iter = stmt.query_map([], |row| {
            Ok(Handle {
                id: row.get(0)?,
                identifier: row.get(1)?,
                service: row.get(2)?,
            })
        })?;
        
        for handle in handle_iter {
            let handle = handle?;
            self.handle_cache.insert(handle.id, handle);
        }
        
        Ok(())
    }
    
    pub fn get_handle(&self, handle_id: i32) -> Option<&Handle> {
        self.handle_cache.get(&handle_id)
    }
    
    pub fn get_recent_messages(&self, limit: i32) -> SqlResult<Vec<RealMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT ROWID, guid, text, handle_id, date, date_edited, date_retracted, 
                    is_from_me, cache_has_attachments
             FROM message 
             WHERE date IS NOT NULL
             ORDER BY date DESC 
             LIMIT ?"
        )?;
        
        let message_iter = stmt.query_map([limit], |row| {
            self.row_to_message(row)
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    pub fn get_messages_newer_than(&self, min_id: i32) -> SqlResult<Vec<RealMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT ROWID, guid, text, handle_id, date, date_edited, date_retracted, 
                    is_from_me, cache_has_attachments
             FROM message 
             WHERE ROWID > ? AND date IS NOT NULL
             ORDER BY ROWID ASC"
        )?;
        
        let message_iter = stmt.query_map([min_id], |row| {
            self.row_to_message(row)
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    pub fn get_messages_by_ids(&self, message_ids: &[i32]) -> SqlResult<Vec<RealMessage>> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let placeholders = message_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT ROWID, guid, text, handle_id, date, date_edited, date_retracted, 
                    is_from_me, cache_has_attachments
             FROM message 
             WHERE ROWID IN ({})",
            placeholders
        );
        
        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = message_ids.iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        
        let message_iter = stmt.query_map(&params[..], |row| {
            self.row_to_message(row)
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    pub fn messages_exist(&self, message_ids: &[i32]) -> SqlResult<Vec<i32>> {
        if message_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let placeholders = message_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!("SELECT ROWID FROM message WHERE ROWID IN ({})", placeholders);
        
        let mut stmt = self.conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = message_ids.iter()
            .map(|id| id as &dyn rusqlite::ToSql)
            .collect();
        
        let id_iter = stmt.query_map(&params[..], |row| {
            Ok(row.get::<_, i32>(0)?)
        })?;
        
        let mut existing_ids = Vec::new();
        for id in id_iter {
            existing_ids.push(id?);
        }
        
        Ok(existing_ids)
    }
    
    pub fn get_recently_modified_messages(&self, since_timestamp: i64) -> SqlResult<Vec<RealMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT ROWID, guid, text, handle_id, date, date_edited, date_retracted, 
                    is_from_me, cache_has_attachments
             FROM message 
             WHERE (date_edited > ? OR date_retracted > ?) AND date_edited IS NOT NULL
             ORDER BY COALESCE(date_edited, date_retracted) DESC"
        )?;
        
        let message_iter = stmt.query_map([since_timestamp, since_timestamp], |row| {
            self.row_to_message(row)
        })?;
        
        let mut messages = Vec::new();
        for message in message_iter {
            messages.push(message?);
        }
        
        Ok(messages)
    }
    
    fn row_to_message(&self, row: &Row) -> SqlResult<RealMessage> {
        Ok(RealMessage {
            id: row.get(0)?,
            guid: row.get(1)?,
            text: row.get(2)?,
            handle_id: row.get(3)?,
            date: row.get(4)?,
            date_edited: row.get(5)?,
            date_retracted: row.get(6)?,
            is_from_me: row.get::<_, i32>(7)? != 0,
            cache_has_attachments: row.get::<_, i32>(8)? != 0,
        })
    }
    
    pub fn get_message_count(&self) -> SqlResult<i32> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM message")?;
        let count: i32 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }
}
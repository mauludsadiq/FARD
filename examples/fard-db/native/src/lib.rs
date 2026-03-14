//! fard_db_native — SQLite KV store with text-pointer C ABI for FARD FFI

use std::sync::Mutex;
use rusqlite::{Connection, params};

static DB: Mutex<Option<Connection>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn fdb_open(path: *const std::os::raw::c_char) -> i64 {
    let path_str = unsafe {
        if path.is_null() { return -1; }
        std::ffi::CStr::from_ptr(path).to_string_lossy().to_string()
    };
    let mut db = DB.lock().unwrap();
    match Connection::open(&path_str) {
        Ok(conn) => {
            let _ = conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value TEXT NOT NULL);"
            );
            *db = Some(conn);
            1
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn fdb_set(
    key: *const std::os::raw::c_char,
    val: *const std::os::raw::c_char,
) -> i64 {
    let key_str = unsafe { std::ffi::CStr::from_ptr(key).to_string_lossy().to_string() };
    let val_str = unsafe { std::ffi::CStr::from_ptr(val).to_string_lossy().to_string() };
    let db = DB.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        match conn.execute(
            "INSERT OR REPLACE INTO kv (key, value) VALUES (?1, ?2)",
            params![key_str, val_str],
        ) {
            Ok(_) => 1,
            Err(_) => -1,
        }
    } else { -1 }
}

// Returns pointer to static thread-local buffer — valid until next call
static RESULT_BUF: Mutex<Option<std::ffi::CString>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn fdb_get(key: *const std::os::raw::c_char) -> i64 {
    let key_str = unsafe { std::ffi::CStr::from_ptr(key).to_string_lossy().to_string() };
    let db = DB.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        let result: rusqlite::Result<String> = conn.query_row(
            "SELECT value FROM kv WHERE key = ?1",
            params![key_str],
            |row| row.get(0),
        );
        match result {
            Ok(val) => {
                drop(db);
                let cs = std::ffi::CString::new(val).unwrap_or_default();
                let ptr = cs.as_ptr() as i64;
                *RESULT_BUF.lock().unwrap() = Some(cs);
                ptr
            }
            Err(_) => 0,
        }
    } else { 0 }
}

#[no_mangle]
pub extern "C" fn fdb_delete(key: *const std::os::raw::c_char) -> i64 {
    let key_str = unsafe { std::ffi::CStr::from_ptr(key).to_string_lossy().to_string() };
    let db = DB.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        match conn.execute("DELETE FROM kv WHERE key = ?1", params![key_str]) {
            Ok(n) => n as i64,
            Err(_) => -1,
        }
    } else { -1 }
}

#[no_mangle]
pub extern "C" fn fdb_count() -> i64 {
    let db = DB.lock().unwrap();
    if let Some(conn) = db.as_ref() {
        conn.query_row("SELECT COUNT(*) FROM kv", [], |r| r.get(0)).unwrap_or(-1)
    } else { -1 }
}

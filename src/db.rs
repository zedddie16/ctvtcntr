use chrono::{NaiveDate, Utc};
use duckdb::{params, Connection, Result};
use tracing::info;

use crate::logic::Usage;

/// Initialize the database and create the table if it doesn't exist
pub fn ensure_table_exists(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS window_log (
            id BIGINT PRIMARY KEY AUTOINCREMENT,
            window_name VARCHAR NOT NULL,
            window_sub_name VARCHAR NOT NULL,
            class VARCHAR NOT NULL 
        )",
        [],
    )?;
    info!("table ensured");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS window_date_log (
            id BIGINT NOT NULL REFERENCES window_log(id),
            ts TIMESTAMP NOT NULL,
            dur INTEGER NOT NULL,
            PRIMARY KEY (id, ts)
        )",
        [],
    )?;
    Ok(())
}

/// Log application activity
/// If a record for the app_name and today's date exists, its usage_time_seconds is incremented
/// Otherwise, a new record is inserted
pub fn log_activity(conn: &Connection, window_name: &str, usage_increment_secs: u64) -> Result<()> {
    let today_naive: NaiveDate = Utc::now().date_naive();

    let sql = "
        INSERT INTO activity_log (date, window_name, usage_time_secs)
        VALUES (?, ?, ?)
        ON CONFLICT (date, window_name) DO UPDATE SET
        usage_time_secs = activity_log.usage_time_secs + excluded.usage_time_secs;
    ";

    conn.execute(sql, params![today_naive, window_name, usage_increment_secs])?;

    info!(
        "Upsert: Date: {}, App: {}, Usage Increment: {}s",
        today_naive, window_name, usage_increment_secs
    );
    Ok(())
}

/// Query and display all records (testing)
pub fn print_all_records(conn: &Connection) -> Result<()> {
    println!("\n--- All Records ---");
    let mut stmt = conn.prepare(
        "SELECT date, window_name, usage_time_secs FROM activity_log ORDER BY date, window_name",
    )?;
    let records_iter = stmt.query_map([], |row| {
        Ok(Usage {
            date: row.get(0)?,
            window_name: row.get(1)?,
            usage_time_secs: row.get(2)?,
        })
    })?;

    for record_result in records_iter {
        match record_result {
            Ok(record) => {
                println!(
                    "Date: {}, App: {}, Total Usage: {}s",
                    record.date, record.window_name, record.usage_time_secs
                );
            }
            Err(e) => eprintln!("Error reading row: {}", e),
        }
    }
    println!("--------------------");
    Ok(())
}

/// Query usage for a specific app on a specific date
pub fn get_usage_for_app_on_date(
    conn: &Connection,
    window_name: &str,
    date: NaiveDate,
) -> Result<Option<i32>> {
    let mut stmt = conn
        .prepare("SELECT usage_time_secs FROM activity_log WHERE window_name = ? AND date = ?")?;
    let mut rows = stmt.query(params![window_name, date])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

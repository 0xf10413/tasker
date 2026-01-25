use rusqlite::{Connection, Error};

const SQLITE_URL: &str = "./tasks.db";

pub trait SqlConnectionFactory: Send + Sync {
    fn open(&self) -> Result<Connection, Error>;
}

pub struct SqliteConnectionFactory;

impl SqlConnectionFactory for SqliteConnectionFactory {
    fn open(&self) -> Result<Connection, Error> {
        Connection::open(SQLITE_URL)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::io;

    use tempfile::{TempDir, tempdir};

    pub struct TempDirSqliteConnectionFactory {
        tempdir: TempDir,
    }

    impl TempDirSqliteConnectionFactory {
        pub fn new() -> Result<Self, io::Error> {
            Ok(TempDirSqliteConnectionFactory {
                tempdir: tempdir()?,
            })
        }
    }

    impl SqlConnectionFactory for TempDirSqliteConnectionFactory {
        fn open(&self) -> Result<Connection, Error> {
            let full_path = format!(
                "{}/tasks.db",
                self.tempdir
                    .path()
                    .to_str()
                    .expect("Path should be OK as generated internally")
            );
            Ok(Connection::open(full_path)?)
        }
    }
}

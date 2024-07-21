use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection, Result};
use crate::todo::TodoEntry;

pub struct Database {
    pub connection: Connection,
}

impl Database {
    pub fn new(db_file: &str) -> Result<Database> {
        let connection = Connection::open(db_file)?;
        connection.execute("CREATE TABLE IF NOT EXISTS ENTRIES(
                                ID INTEGER PRIMARY KEY,
                                TITLE TEXT NOT NULL,
                                DESCRIPTION TEXT NOT NULL,
                                CREATION_DATE TEXT NOT NULL,
                                DUE_DATE TEXT,
                                ISDONE INTEGER NOT NULL)", params![])?;
        Ok(
            Database { connection }
        )
    }

    pub fn get_connection(&self) -> &Connection {
        &self.connection
    }

    pub fn get_all_entries(&mut self) -> Result<Vec<TodoEntry>> {
        let mut command = self.connection.prepare("SELECT ID, TITLE, DESCRIPTION, CREATION_DATE, DUE_DATE, ISDONE
                                     FROM ENTRIES")?;

        let result = command.query_map(params![],
                                       |row| {
                                               TodoEntry::from_row(row)
                                       })?
                            .map(|entry| entry.unwrap())
                            .collect();

        Ok(result)
    }

    pub fn get_entries_by_range(&mut self, start_index: usize, number_of_entries: usize) -> Result<Vec<TodoEntry>> {
        let mut command = self.connection.prepare("SELECT ID, TITLE, DESCRIPTION, CREATION_DATE, DUE_DATE, ISDONE
                                     FROM ENTRIES
                                     LIMIT ?1
                                     OFFSET ?2")?;

        let result = command.query_map(params![number_of_entries, start_index],
                                       |row| {
                                               TodoEntry::from_row(row)
                                       })?
                            .map(|entry| entry.unwrap())
                            .collect::<Vec<TodoEntry>>();
        Ok(result)
    }

    pub fn update_database(&mut self, entry: &TodoEntry) -> Result<()> {
        self.connection.execute("UPDATE ENTRIES
                                     SET TITLE = ?1,
                                     DESCRIPTION = ?2
                                     WHERE ID = ?3
                                    ",
                                params![entry.title, entry.description, entry.id]
        )?;
        Ok(())
    }

    pub fn insert_entry<T: Into<String>>(&mut self, title: T, description: T, due_date: Option<NaiveDate>) -> Result<TodoEntry> {
        let creation_date = Local::now().date_naive();
        let title = title.into();
        let description = description.into();
        self.connection.execute("INSERT INTO ENTRIES(TITLE, DESCRIPTION, CREATION_DATE, DUE_DATE, ISDONE)
                                     VALUES(?1, ?2, ?3, ?4, ?5)",
                                params![
                                    title.clone(),
                                    description.clone(),
                                    creation_date,
                                    due_date,
                                    false
                                ]
        )?;
        let mut id = self.connection.prepare("SELECT last_insert_rowid()")?;
        let id: usize = id.query_row(params![], |row| {
            Ok(
                row.get(0)?
            )
        })?;

        Ok(
            TodoEntry {
            id,
            title,
            description,
            creation_date,
            due_date,
            is_done: false
            }
        )
    }

    pub fn remove_from_database(&mut self, entry: &TodoEntry) -> Result<()> {
        self.connection.execute("DELETE FROM ENTRIES
                                     WHERE ID = ?1", params![entry.id])?;
        Ok(())
    }
}

#![allow(dead_code)]
use chrono::prelude::*;
use std::fmt::Display;
use rusqlite::{Result, Row};

#[derive(Debug)]
pub struct TodoEntry {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub creation_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub is_done: bool,
}

impl Display for TodoEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, Title: {}, Status: {}\nDescription: {}",
               self.id,
               self.title,
               if self.is_done {"Done"} else {"Pending"},
               self.description)
    }
}

impl TodoEntry {
    pub fn new<T: Into<String>>(id: usize, title: T, description: T, creation_date: NaiveDate, due_date: Option<NaiveDate>) -> TodoEntry {
        TodoEntry {
            id,
            title: title.into(),
            description: description.into(),
            creation_date,
            due_date,
            is_done: false,
        }
    }

    pub fn change_title<T>(&mut self, new_title: T) where T: Into<String> {
        self.title = new_title.into();
    }

    pub fn change_description<T>(&mut self, new_description: T) where T: Into<String> {
        self.description = new_description.into();
    }

    pub fn toggle_done(&mut self) {
        self.is_done = !self.is_done;
    }

    pub fn from_row(row: &Row) -> Result<TodoEntry> {
        Ok(
            TodoEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                creation_date: row.get(3)?,
                due_date: row.get(4)?,
                is_done: row.get(5)?,
            }
        )
    }

    pub fn show_info(&self) {
        println!(
            "ID: {}\n\
             Title: {}\n\
             Description: {}\n\
             Due Date: {}\n\
             Status: {}\n",
            self.id,
            self.title,
            self.description,
            if self.due_date.is_some() {self.due_date.unwrap().to_string()} else {"No Due Date".to_owned()},
            if self.is_done {"Done"} else {"Pending"}
        );
    }

}

pub struct TodoList {
    pub list: Vec<TodoEntry>
}

impl TodoList {
    pub fn new() -> TodoList {
        TodoList { list: Vec::<TodoEntry>::new() }
    }

    pub fn from_vec(list: Vec<TodoEntry>) -> TodoList {
        TodoList { list}
    }

    pub fn get_mut_by_id(&mut self, id: usize) -> Option<&mut TodoEntry> {
        self.list.iter_mut().find(|entry| entry.id == id)
    }
}

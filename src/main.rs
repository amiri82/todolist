#![allow(unused)]
use chrono::prelude::*;
use inquire::{required, DateSelect, Select, Text};
use std::{fmt::{write, Display}, io::{stdin, stdout, Read, Write}, process::Termination};
use rusqlite::{params, Connection, Result};

mod todo;
mod database;

use todo::{TodoEntry, TodoList};
use database::Database;

enum Prompts {
    MainMenu,
    AddEntry,
    SelectEntry(Vec<TodoEntry>),
    EntryEditMenu(TodoEntry),
    ChangeTitle(TodoEntry),
    ChangeDescription(TodoEntry),
    InfoPage(TodoEntry),
    Exit
}

enum MainMenuOptions {
    AddEntry,
    ShowEntries,
    Exit
}

impl Display for MainMenuOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MainMenuOptions::AddEntry => write!(f, "Add Entry"),
            MainMenuOptions::ShowEntries => write!(f, "Show and Edit Entries"),
            MainMenuOptions::Exit => write!(f, "Exit"),
        }
    }
}

enum InfoPageOptions {
    EditEntry,
    DeleteEntry,
    ReturnToMainMenu,
}

impl Display for InfoPageOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfoPageOptions::EditEntry => write!(f, "Edit Entry"),
            InfoPageOptions::ReturnToMainMenu => write!(f, "Return To MainMenu"),
            InfoPageOptions::DeleteEntry => write!(f, "Delete Entry"),
        }
    }
}

enum EntryEditOptions {
    ChangeTitle,
    ChangeDescription,
    ToggleDone,
    ReturnToMainMenu,
}

impl Display for EntryEditOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryEditOptions::ChangeTitle => write!(f, "Change Title"),
            EntryEditOptions::ChangeDescription => write!(f, "Change Description"),
            EntryEditOptions::ToggleDone => write!(f, "Toggle Done"),
            EntryEditOptions::ReturnToMainMenu => write!(f, "Return To Main Menu"),
        }
    }
}


impl Prompts {

    fn prompt(self) -> Operations {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        stdout().flush().unwrap();
        match self {
            Prompts::MainMenu => {
                let options: Vec<MainMenuOptions> = vec![
                    MainMenuOptions::AddEntry,
                    MainMenuOptions::ShowEntries,
                    MainMenuOptions::Exit
                ];
                let choice = Select::new("Select The Operation:", options)
                    .prompt()
                    .unwrap();
                match choice {
                    MainMenuOptions::AddEntry => Operations::SkipToPrompt(Prompts::AddEntry),
                    MainMenuOptions::ShowEntries => Operations::ListEntries,
                    MainMenuOptions::Exit => Operations::Exit
                }
            },
            Prompts::AddEntry => {
                let title: String = Text::new("Enter title:")
                    .with_validator(required!("This field is required!"))
                    .prompt()
                    .unwrap();

                let description: String = Text::new("Enter description:")
                    .with_validator(required!("This field is required"))
                    .prompt()
                    .unwrap();

               let due_date: Option<NaiveDate> = DateSelect::new("Select due date if necessary:")
                .prompt_skippable()
                .unwrap();

                Operations::AddEntry { title, description , due_date}
            },
            Prompts::SelectEntry(entries) => {
                if entries.is_empty() {
                    println!("No Entries");
                    Operations::SkipToPrompt(Prompts::MainMenu)
                } else {
                    let choice: Option<TodoEntry> = Select::new("Select the wanted entry:", entries)
                        .prompt_skippable()
                        .unwrap();
                    match choice {
                        Some(entry) => Operations::SkipToPrompt(Prompts::InfoPage(entry)),
                        None => Operations::SkipToPrompt(Prompts::MainMenu),
                    }
                }
            },
            Prompts::EntryEditMenu(entry) => {
                let options: Vec<EntryEditOptions> = vec![
                    EntryEditOptions::ChangeTitle,
                    EntryEditOptions::ChangeDescription,
                    EntryEditOptions::ToggleDone,
                    EntryEditOptions::ReturnToMainMenu,
                ];
               
                let choice: EntryEditOptions = Select::new("Select the apropriate option:", options)
                    .prompt()
                    .unwrap();
               
                match choice {
                    EntryEditOptions::ChangeTitle => Operations::SkipToPrompt(Prompts::ChangeTitle(entry)),
                    EntryEditOptions::ChangeDescription => Operations::SkipToPrompt(Prompts::ChangeDescription(entry)),
                    EntryEditOptions::ToggleDone => Operations::ToggleDone(entry),
                    EntryEditOptions::ReturnToMainMenu => Operations::SkipToPrompt(Prompts::MainMenu),
                }
            },
            Prompts::InfoPage(entry) => {
                entry.show_info();

                let options : Vec<InfoPageOptions> = vec![
                    InfoPageOptions::EditEntry,
                    InfoPageOptions::DeleteEntry,
                    InfoPageOptions::ReturnToMainMenu
                ];

                let choice: InfoPageOptions = Select::new("Select Operation: ", options)
                    .prompt()
                    .unwrap();

                match choice {
                    InfoPageOptions::EditEntry => Operations::SkipToPrompt(Prompts::EntryEditMenu(entry)),
                    InfoPageOptions::DeleteEntry => Operations::DeleteEntry(entry),
                    InfoPageOptions::ReturnToMainMenu => Operations::SkipToPrompt(Prompts::MainMenu),
                }
            },
            Prompts::ChangeTitle(entry) => {
                let title: String = Text::new("Enter new title")
                    .with_validator(required!("This field is required"))
                    .prompt()
                    .unwrap();

                Operations::ChangeTitle(entry, title)
            },
            Prompts::ChangeDescription(entry) => {
                let desc: String = Text::new("Enter new description:")
                    .with_validator(required!("This field is required"))
                    .prompt()
                    .unwrap();
                Operations::ChangeDescription(entry, desc)
            },
            Prompts::Exit => Operations::Exit
        }
    }

    fn is_exit(&self) -> bool {
        if let &Prompts::Exit = &self {
            true
        } else {
            false
        }
    }

}

enum Operations {
    SkipToPrompt(Prompts),
    ListEntries,
    AddEntry {
        title: String,
        description: String,
        due_date: Option<NaiveDate>,
    },
    ChangeDescription(TodoEntry, String),
    ChangeTitle(TodoEntry, String),
    DeleteEntry(TodoEntry),
    ToggleDone(TodoEntry),
    Exit
}

impl Operations {
    fn operate(self, database: &mut Database, todo_list: &mut TodoList) -> Prompts {
        match self {
            Operations::SkipToPrompt(new_prompt) => {
                new_prompt
            },
            Operations::ListEntries => Prompts::SelectEntry(database.get_all_entries().expect("Couldn't get entries from database")),
            Operations::AddEntry { title, description, due_date } => {
                let new_entry: TodoEntry = database.insert_entry(title, description, due_date).expect("Couldn't insert entry into database");
                todo_list.list.push(new_entry);
                Prompts::MainMenu
            },
            Operations::ChangeDescription(mut entry, new_description ) => {
                entry.change_description(new_description.clone());
                todo_list.get_mut_by_id(entry.id).unwrap().change_description(new_description);
                database.update_database(&entry);
                Prompts::MainMenu
            },
            Operations::ChangeTitle(mut entry, new_title) => {
                entry.change_title(new_title.clone());
                todo_list.get_mut_by_id(entry.id).unwrap().change_title(new_title);
                database.update_database(&entry);
                Prompts::MainMenu
            },
            Operations::ToggleDone(mut entry) => {
                entry.toggle_done();
                todo_list.get_mut_by_id(entry.id).unwrap().toggle_done();
                database.update_database(&entry);
                Prompts::MainMenu
            },
            Operations::DeleteEntry(entry) => {
                database.remove_from_database(&entry).expect("Couldn't remove from database");
                let pos: usize = todo_list.list.iter().position(|ent| ent.id == entry.id).unwrap();
                todo_list.list.remove(pos);
                Prompts::MainMenu
            },
            Operations::Exit => Prompts::Exit,
        }
    }
   
}


fn main() {
    let mut database: Database = Database::new("entries.db").expect("Couldn't open databae");
    let mut todo_list = database.get_all_entries().expect("Couldn't intialize the list from database");
    let mut todo_list = TodoList::from_vec(todo_list);
    let mut current_prompt = Prompts::MainMenu;
    let mut operation: Operations;
    while !current_prompt.is_exit() {
        operation = current_prompt.prompt();
        current_prompt = operation.operate(&mut database, &mut todo_list);
    }
}

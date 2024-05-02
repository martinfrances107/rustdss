use rustdss_data::{Command, Key, RespData};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::thread;

mod base_logic;
mod db_logic;

pub type DatabaseId = String;

pub type Message = (DatabaseId, Command, Sender<RespData>);
// This is the stateful part of the application
pub struct Core {
    sender: Sender<Message>,
}

pub struct CoreState {
    keyval: HashMap<Key, RespData>,
}

impl Core {
    fn create_database(db_id: String) -> Sender<(Command, Sender<RespData>)> {
        let (db_sender, db_reciever) = channel::<(Command, Sender<RespData>)>();

        thread::spawn(move || {
            let mut db_state = CoreState {
                keyval: HashMap::new(),
            };
            loop {
                if let Ok(msg) = db_reciever.recv() {
                    let (cmd, responder) = msg;
                    let response = base_logic::core_logic(&mut db_state, cmd);
                    responder
                        .send(response)
                        .unwrap_or_else(|_| panic!("[core::{}] can't reply to messages", db_id));
                } else {
                    println!("[core::{}] db_core died/dropped?", db_id);
                    break;
                }
            }
        });

        db_sender
    }
    pub fn start() -> Self {
        // Could do something interesting using a threadpool - key-hash sharding for example
        println!("[core] starting core");
        let (sender, reciever) = channel::<Message>();

        // Each database get's it's own thread
        thread::spawn(move || {
            // This thread needs to keep track of all the databases available
            // each database needs it's own CoreState
            let mut databases: HashMap<DatabaseId, Sender<(Command, Sender<RespData>)>> =
                HashMap::new();

            databases.insert("default".into(), Self::create_database("default".into()));

            loop {
                if let Ok(msg) = reciever.recv() {
                    let (database_id, cmd, responder) = msg;

                    if let Some(db_sender) = databases.get(&database_id) {
                        db_sender
                            .send((cmd, responder))
                            .expect("[core::router] Can't send to database");
                    } else {
                        let newdb_sender = Self::create_database(database_id.clone());
                        databases.insert(database_id, newdb_sender.clone());

                        newdb_sender
                            .send((cmd, responder))
                            .expect("[core::router] Can't send to new database");
                    }
                } else {
                    println!("[core] death");
                }
            }
        });
        Self { sender }
    }

    pub fn get_sender(&self) -> Sender<Message> {
        self.sender.clone()
    }
}

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use diesel::Insertable;

use dotenv::dotenv;
use std::env;

use crate::db_schema::{states_log, self};
use crate::state::State;

#[derive(Insertable)]
#[table_name="states_log"]
pub struct StateLog {
    pub new_state: String,
}

pub fn establish_connection() -> SqliteConnection {

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    SqliteConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn insert_state_log(conn: &SqliteConnection, state: State) {
    diesel::insert_into(db_schema::states_log::table)
        .values(&StateLog {
            new_state: state.to_str().to_string(),
        })
        .execute(conn)
        .expect("Could not post to database.");
}

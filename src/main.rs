use std::env;

use actix_web::{http::Error, web, App, HttpResponse, HttpServer, Result};
use serde::{Deserialize, Serialize};
use libsql::{Builder, Database, Value};
use dotenvy::dotenv;


// the struct for a todo item
#[derive(Serialize)]
struct Todo {
    task: String,
}

// the input to the create_todos handler
#[derive(Deserialize, Serialize, Debug)]
struct CreateTodo {
    task: String,
}

async fn index() -> Result<String> {
    Ok(format!("Hello, Actix! ❤︎ Turso"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)).route("/todos", web::post().to(create_todo)).route("/todos", web::get().to(get_todos)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

// Returns a database connection
async fn connection() -> Database {
    dotenv().expect(".env file not found");

    let db_file = env::var("LOCAL_DB").unwrap();

    let auth_token = env::var("TURSO_AUTH_TOKEN").unwrap_or_else(|_| {
        println!("Using empty token since TURSO_AUTH_TOKEN was not set");
        "".to_string()
    });

    let url = env::var("TURSO_DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("Using http://localhost:8080 TURSO_DATABASE_URL was not set");
            "http://localhost:8080".to_string()
        })
        .replace("libsql", "https");

    let db = Builder::new_remote_replica(db_file, url, auth_token)
    .read_your_writes(true)
    .build()
    .await
    .unwrap();

    let conn = db.connect().unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS todos(task varchar non null)", ())
        .await
        .unwrap();

    db.sync().await.unwrap();

    db
}

// Gets all tasks from the todo table
async fn get_todos() -> Result<HttpResponse, Error> {
    let db = connection().await;
    let conn = db.connect().unwrap();

    let mut results = conn.query("SELECT * FROM todos", ()).await.unwrap();

    let mut todos: Vec<Todo> = Vec::new();

    while let Some(row) = results.next().await.unwrap() {
        let todo: Todo = Todo {
            task: row.get(0).unwrap(),
        };
        todos.push(todo);
    }

    Ok(HttpResponse::Ok().json(todos))
}

// Creates a new task in the todo table
async fn create_todo(payload: web::Json<CreateTodo>) -> Result<HttpResponse, Error> {
    let todo = Todo { task: payload.task.clone() };

    let db = connection().await;
    let conn = db.connect().unwrap();

    let _ = conn
        .query("INSERT into todos values (?1)", vec![Value::from(todo.task.clone())])
        .await;

    db.sync().await.unwrap();

    Ok(HttpResponse::Ok().json(todo))
}

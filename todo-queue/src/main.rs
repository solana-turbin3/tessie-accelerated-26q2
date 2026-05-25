mod models;
mod queue;
mod storage;

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use models::Todo;
use queue::Queue;
use storage::{load_from_file, save_to_file};

fn main() {
    let mut queue: Queue<Todo> = load_from_file("todos.bin");
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("add") => {
            let Some(description) = args.get(2) else {
                println!("Usage:");
                println!("  todo-queue add \"Buy groceries\"");
                return;
            };

            let todo = Todo {
                id: next_id(&queue),
                description: description.clone(),
                created_at: current_timestamp(),
            };

            queue.enqueue(todo);
            save_to_file(&queue, "todos.bin");
            println!("Task added.");
        }
        Some("list") => {
            if queue.is_empty() {
                println!("No pending tasks.");
                return;
            }

            for todo in queue.iter() {
                println!("#{}: {}", todo.id, todo.description);
            }
        }
        Some("done") => match queue.dequeue() {
            Some(todo) => {
                save_to_file(&queue, "todos.bin");
                println!("Completed #{}: {}", todo.id, todo.description);
            }
            None => {
                println!("No pending tasks.");
            }
        },
        _ => {
            println!("Usage:");
            println!("  todo-queue add \"Buy groceries\"");
            println!("  todo-queue list");
            println!("  todo-queue done");
        }
    }
}

fn next_id(queue: &Queue<Todo>) -> u64 {
    queue.iter().map(|todo| todo.id).max().unwrap_or(0) + 1
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

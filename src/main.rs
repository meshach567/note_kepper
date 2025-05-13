use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use askama::Template;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    notes: &'a [Note],
}

#[derive(Template)]
#[template(path = "new_note.html")]
struct NewNoteTemplate;

#[derive(Debug, Clone)]
struct Note {
    id: usize,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct NewNote {
    title: String,
    body: String,
}

type SharedState = Arc<Mutex<Vec<Note>>>;

#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
    .route("/", get(show_notes))
    .route("/new", get(new_note_form).post(create_note)) // Add `.post(create_note)`
    .route("/create", post(create_note))
    .route("/delete/:id", get(delete_note))
    .with_state(state.clone());

    println!("🚀 Running on http://localhost:3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn show_notes(state: axum::extract::State<SharedState>) -> impl IntoResponse {
    let notes = state.lock().unwrap();
    let template = IndexTemplate { notes: &notes };
    Html(template.render().unwrap())
}

async fn new_note_form() -> impl IntoResponse {
    Html(NewNoteTemplate.render().unwrap())
}

async fn create_note(
    state: axum::extract::State<SharedState>,
    Form(data): Form<NewNote>,
) -> impl IntoResponse {
    let mut notes = state.lock().unwrap();
    let id = notes.len() + 1; // Simple ID generation
    notes.push(Note {
        id,
        title: data.title,
        body: data.body,
    });
    axum::response::Redirect::to("/")
}

async fn delete_note(
    state: axum::extract::State<SharedState>,
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> impl IntoResponse {
    let mut notes = state.lock().unwrap();
    if let Some(pos) = notes.iter().position(|note| note.id == id) {
        notes.remove(pos);
        println!("Note with ID {} deleted.", id);
    } else {
        println!("Note with ID {} not found.", id);
    }
    axum::response::Redirect::to("/")
}

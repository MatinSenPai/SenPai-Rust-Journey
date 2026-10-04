use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, ID};
use std::collections::HashMap;
use std::sync::Mutex;

/// A note, exposed to GraphQL clients as an object type. `SimpleObject`
/// derives field resolvers for every public field automatically — no `impl`
/// block needed when a type is "just data", the GraphQL analogue of
/// `#[derive(Serialize)]` on a JSON response struct.
#[derive(Clone, SimpleObject)]
pub struct Note {
    pub id: ID,
    pub title: String,
    pub body: String,
}

#[derive(Default)]
pub struct NotesState {
    next_id: u64,
    notes: HashMap<String, Note>,
}

pub type NotesSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Builds a ready-to-execute schema with a fresh, empty `NotesState`
/// attached as request-independent shared data (`.data(...)`).
pub fn build_schema() -> NotesSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(Mutex::new(NotesState::default()))
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Look up a single note by id. Returns `None` (GraphQL `null`) rather
    /// than an error when the id doesn't exist — unlike gRPC's
    /// `Status::not_found`, a GraphQL field that can legitimately be absent
    /// is usually typed as nullable instead of erroring.
    async fn note(&self, ctx: &Context<'_>, id: ID) -> Option<Note> {
        let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
        todo!("look up `id.as_str()` in `state.notes` and return a cloned Option<Note>: {{}}")
    }

    /// Every note currently stored, in no particular order.
    async fn notes(&self, ctx: &Context<'_>) -> Vec<Note> {
        let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
        todo!("collect state.notes.values().cloned() into a Vec<Note>: {{}}")
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Creates a note and returns it — the GraphQL mutation equivalent of
    /// the gRPC lesson's `create_note` RPC and axum's `POST /notes` handler.
    async fn create_note(&self, ctx: &Context<'_>, title: String, body: String) -> Note {
        let mut state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
        todo!(
            "assign the next id from state.next_id (incrementing it), build a Note, \
             insert it into state.notes, and return a clone of it: {{}}"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn creates_and_fetches_a_note_by_id() {
        let schema = build_schema();

        let create = schema
            .execute(
                r#"mutation {
                    createNote(title: "GraphQL notes", body: "queries ask for exact fields") {
                        id
                        title
                        body
                    }
                }"#,
            )
            .await;
        assert!(create.errors.is_empty(), "{:?}", create.errors);

        let data = create.data.into_json().unwrap();
        let id = data["createNote"]["id"].as_str().unwrap().to_string();
        assert_eq!(data["createNote"]["title"], "GraphQL notes");

        let fetch = schema
            .execute(format!(
                r#"query {{ note(id: "{id}") {{ id title body }} }}"#
            ))
            .await;
        assert!(fetch.errors.is_empty(), "{:?}", fetch.errors);

        let fetched = fetch.data.into_json().unwrap();
        assert_eq!(fetched["note"]["id"], id);
        assert_eq!(fetched["note"]["title"], "GraphQL notes");
    }

    #[tokio::test]
    async fn note_returns_null_for_a_missing_id() {
        let schema = build_schema();

        let result = schema
            .execute(r#"query { note(id: "does-not-exist") { id } }"#)
            .await;

        assert!(result.errors.is_empty(), "{:?}", result.errors);
        let data = result.data.into_json().unwrap();
        assert!(data["note"].is_null());
    }

    #[tokio::test]
    async fn notes_query_returns_every_field_a_client_asked_for() {
        let schema = build_schema();

        schema
            .execute(r#"mutation { createNote(title: "first", body: "") { id } }"#)
            .await;
        schema
            .execute(r#"mutation { createNote(title: "second", body: "") { id } }"#)
            .await;

        // Note the query below asks only for `title` — not `id` or `body` —
        // which is the entire point of GraphQL: the client picks the shape.
        let result = schema.execute(r#"query { notes { title } }"#).await;
        assert!(result.errors.is_empty(), "{:?}", result.errors);

        let data = result.data.into_json().unwrap();
        let notes = data["notes"].as_array().unwrap();
        assert_eq!(notes.len(), 2);
        assert!(notes[0].get("id").is_none());
        let mut titles: Vec<&str> = notes.iter().map(|n| n["title"].as_str().unwrap()).collect();
        titles.sort_unstable();
        assert_eq!(titles, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn each_created_note_gets_a_distinct_id() {
        let schema = build_schema();

        let first = schema
            .execute(r#"mutation { createNote(title: "a", body: "") { id } }"#)
            .await
            .data
            .into_json()
            .unwrap();
        let second = schema
            .execute(r#"mutation { createNote(title: "b", body: "") { id } }"#)
            .await
            .data
            .into_json()
            .unwrap();

        assert_ne!(first["createNote"]["id"], second["createNote"]["id"]);
    }
}

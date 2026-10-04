use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, ID};
use std::collections::HashMap;
use std::sync::Mutex;

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

pub fn build_schema() -> NotesSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(Mutex::new(NotesState::default()))
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn note(&self, ctx: &Context<'_>, id: ID) -> Option<Note> {
        let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
        state.notes.get(id.as_str()).cloned()
    }

    async fn notes(&self, ctx: &Context<'_>) -> Vec<Note> {
        let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
        state.notes.values().cloned().collect()
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_note(&self, ctx: &Context<'_>, title: String, body: String) -> Note {
        let mut state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();

        let id = state.next_id.to_string();
        state.next_id += 1;

        let note = Note {
            id: ID::from(id.clone()),
            title,
            body,
        };
        state.notes.insert(id, note.clone());
        note
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

use std::collections::HashMap;
use std::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod notes {
    tonic::include_proto!("notes");
}

use notes::{CreateNoteRequest, GetNoteRequest, ListNotesRequest, ListNotesResponse, Note};

#[derive(Default)]
pub struct NotesServiceImpl {
    state: Mutex<NotesState>,
}

#[derive(Default)]
struct NotesState {
    next_id: u64,
    notes: HashMap<String, Note>,
}

impl NotesServiceImpl {
    pub fn new() -> Self {
        Self::default()
    }
}

#[tonic::async_trait]
impl notes::notes_service_server::NotesService for NotesServiceImpl {
    async fn create_note(
        &self,
        request: Request<CreateNoteRequest>,
    ) -> Result<Response<Note>, Status> {
        let req = request.into_inner();
        let mut state = self.state.lock().unwrap();

        let id = state.next_id.to_string();
        state.next_id += 1;

        let note = Note {
            id: id.clone(),
            title: req.title,
            body: req.body,
        };
        state.notes.insert(id, note.clone());

        Ok(Response::new(note))
    }

    async fn get_note(&self, request: Request<GetNoteRequest>) -> Result<Response<Note>, Status> {
        let req = request.into_inner();
        let state = self.state.lock().unwrap();

        match state.notes.get(&req.id) {
            Some(note) => Ok(Response::new(note.clone())),
            None => Err(Status::not_found(format!("note {} not found", req.id))),
        }
    }

    async fn list_notes(
        &self,
        _request: Request<ListNotesRequest>,
    ) -> Result<Response<ListNotesResponse>, Status> {
        let state = self.state.lock().unwrap();
        let notes: Vec<Note> = state.notes.values().cloned().collect();
        Ok(Response::new(ListNotesResponse { notes }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notes::notes_service_server::NotesService;

    #[tokio::test]
    async fn creates_and_fetches_a_note() {
        let service = NotesServiceImpl::new();

        let created = service
            .create_note(Request::new(CreateNoteRequest {
                title: "Rust notes".to_string(),
                body: "traits are structural, not nominal".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert!(!created.id.is_empty());
        assert_eq!(created.title, "Rust notes");

        let fetched = service
            .get_note(Request::new(GetNoteRequest {
                id: created.id.clone(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(fetched, created);
    }

    #[tokio::test]
    async fn get_note_returns_not_found_status_for_a_missing_id() {
        let service = NotesServiceImpl::new();

        let err = service
            .get_note(Request::new(GetNoteRequest {
                id: "does-not-exist".to_string(),
            }))
            .await
            .unwrap_err();

        assert_eq!(err.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn list_notes_returns_every_created_note() {
        let service = NotesServiceImpl::new();

        service
            .create_note(Request::new(CreateNoteRequest {
                title: "first".to_string(),
                body: "".to_string(),
            }))
            .await
            .unwrap();
        service
            .create_note(Request::new(CreateNoteRequest {
                title: "second".to_string(),
                body: "".to_string(),
            }))
            .await
            .unwrap();

        let listed = service
            .list_notes(Request::new(ListNotesRequest {}))
            .await
            .unwrap()
            .into_inner();

        assert_eq!(listed.notes.len(), 2);
        let mut titles: Vec<&str> = listed.notes.iter().map(|n| n.title.as_str()).collect();
        titles.sort_unstable();
        assert_eq!(titles, vec!["first", "second"]);
    }

    #[tokio::test]
    async fn each_created_note_gets_a_distinct_id() {
        let service = NotesServiceImpl::new();

        let first = service
            .create_note(Request::new(CreateNoteRequest {
                title: "a".to_string(),
                body: "".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();
        let second = service
            .create_note(Request::new(CreateNoteRequest {
                title: "b".to_string(),
                body: "".to_string(),
            }))
            .await
            .unwrap()
            .into_inner();

        assert_ne!(first.id, second.id);
    }
}

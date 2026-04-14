use common::{
    id::{NodeId, NoteId},
    note::{CreateNoteRequest, FeedNote, Note, UpdateNoteRequest},
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn fetch_notes(node_id: NodeId) -> Result<Vec<Note>, UiError> {
    let resp = Request::get(&api_url(&format!("/nodes/{node_id}/notes")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_note(node_id: NodeId, req: &CreateNoteRequest) -> Result<Note, UiError> {
    let resp = Request::post(&api_url(&format!("/nodes/{node_id}/notes")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn update_note(note_id: NoteId, req: &UpdateNoteRequest) -> Result<Note, UiError> {
    let resp = Request::patch(&api_url(&format!("/notes/{note_id}")))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn fetch_notes_feed() -> Result<Vec<FeedNote>, UiError> {
    let resp = Request::get(&api_url("/notes/feed"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

#[allow(dead_code)]
pub fn _use_note_id(_: NoteId) {}

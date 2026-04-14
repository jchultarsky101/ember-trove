use common::{
    favorite::{CreateFavoriteRequest, Favorite, ReorderFavoritesRequest},
    id::FavoriteId,
};
use gloo_net::http::Request;

use super::{api_url, parse_json};
use crate::error::UiError;

pub async fn fetch_favorites() -> Result<Vec<Favorite>, UiError> {
    let resp = Request::get(&api_url("/favorites"))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn create_favorite(req: &CreateFavoriteRequest) -> Result<Favorite, UiError> {
    let resp = Request::post(&api_url("/favorites"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

pub async fn delete_favorite(id: FavoriteId) -> Result<(), UiError> {
    let resp = Request::delete(&api_url(&format!("/favorites/{id}")))
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    if resp.ok() {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(UiError::api(status, text))
    }
}

pub async fn reorder_favorites(req: &ReorderFavoritesRequest) -> Result<Vec<Favorite>, UiError> {
    let resp = Request::patch(&api_url("/favorites/reorder"))
        .json(req)
        .map_err(|e| UiError::Parse(e.to_string()))?
        .send()
        .await
        .map_err(|e| UiError::Network(e.to_string()))?;
    parse_json(resp).await
}

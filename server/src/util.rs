use actix_web::http::StatusCode;
use log::error;
use uuid::Uuid;

use crate::{BladeApiError, session::Session};

pub trait CharacterHolder {
    fn get_user_id(&self) -> &Uuid;
}

pub fn get_only_single_character_and_check_permission<T: CharacterHolder>(
    v: Vec<T>,
    session: &Session,
) -> Result<T, BladeApiError> {
    if v.len() > 1 {
        error!("single character expected, found more than one");
        return Err(BladeApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            1,
            100,
        ));
    };

    let first = if let Some(v) = v.into_iter().next() {
        v
    } else {
        return Err(BladeApiError::new(StatusCode::NOT_FOUND, 20000, 2));
    };

    if first.get_user_id() != &session.user_id {
        return Err(BladeApiError::unauthorized());
    };

    Ok(first)
}

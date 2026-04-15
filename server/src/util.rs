use actix_web::http::StatusCode;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use log::error;
use uuid::Uuid;

use crate::{BladeApiError, models::CharacterDbAlone, session::Session};

pub trait CharacterHolder {
    fn get_user_id(&self) -> &Uuid;
}

/// Obtain some character info, while checking said session has permission to get it.
///
/// Will perform a database query, can run inside a transaction, but will not lock for update.
pub async fn check_permission_for_character_and_get_it(
    conn: &mut AsyncPgConnection,
    session: &Session,
    character_id: Uuid,
) -> Result<CharacterDbAlone, BladeApiError> {
    let character_result = {
        use crate::schema::characters::dsl::*;

        characters
            .filter(id.eq(character_id))
            .select(CharacterDbAlone::as_select())
            .load(conn)
            .await
            .unwrap()
    };

    Ok(get_only_single_character_and_check_permission(
        character_result,
        session,
    )?)
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

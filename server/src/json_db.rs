//diesel_json also provide that. It’s exactly the same kind of implementation, except no support for Json (which is uneeded in my case, but I’ll keep my own implementation in case I need some customisation).
use std::fmt::Debug;

use diesel::{
    backend::Backend,
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::Pg,
    serialize::{IsNull, Output, ToSql},
    sql_types::{Json, Jsonb},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

#[derive(FromSqlRow, AsExpression, Debug)]
#[diesel(sql_type = Json, sql_type = Jsonb)]
pub struct JsonDbWrapper<T: Debug>(pub T);

trait JsonOrJsonb {}
impl JsonOrJsonb for Json {}
impl JsonOrJsonb for Jsonb {}

//TODO: specialised implementation for async db, without going though serde_json::Value
impl<T: DeserializeOwned + Debug, DB: Backend, J: JsonOrJsonb> FromSql<J, DB> for JsonDbWrapper<T>
where
    serde_json::Value: FromSql<J, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> diesel::deserialize::Result<Self> {
        let value: Value = FromSql::<J, DB>::from_sql(bytes)?;
        let result: T = match serde_json::from_value(value) {
            Ok(v) => v,
            Err(e) => return Err(format!("failed to deserialize JSON value: {:#?}", e).into()),
        };

        return Ok(JsonDbWrapper(result));
    }
}

impl<T: Serialize + Debug, J: JsonOrJsonb> ToSql<J, Pg> for JsonDbWrapper<T>
where
    serde_json::Value: ToSql<J, Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        let value = match serde_json::to_value(&self.0) {
            Ok(v) => v,
            Err(e) => return Err(format!("failed to serialize JSON value: {:#?}", e).into()),
        };
        ToSql::<J, Pg>::to_sql(&value, &mut out.reborrow())?;
        Ok(IsNull::No)
    }
}

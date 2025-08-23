use crate::db::schemas::BaseFunctions;
use oximod::_error::oximod_error::OxiModError;

pub async fn get_or_create<T: BaseFunctions>(id: String) -> Result<T, OxiModError> {
    match T::find_by_id(id.clone()).await? {
        Some(entity) => Ok(entity),
        None => T::create_with_id(id).await,
    }
}

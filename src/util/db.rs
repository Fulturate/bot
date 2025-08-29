use oximod::Model;
use crate::db::schemas::group::Group;
use crate::db::schemas::user::User;
use crate::util::currency::converter::get_default_currencies;
use crate::util::errors::MyError;

pub async fn create_default_values(id: String, is_user: bool) -> Result<(), MyError> {
    let necessary_codes = get_default_currencies()?;

    let _ = if is_user {
        User::new()
            .user_id(id.clone())
            .convertable_currencies(necessary_codes)
            .save()
            .await
    } else {
        Group::new()
            .group_id(id.clone())
            .convertable_currencies(necessary_codes)
            .save()
            .await
    };

    Ok(())
}
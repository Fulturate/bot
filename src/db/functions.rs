use super::schemas::Group;
use mongodb::bson::{doc, oid::ObjectId};
use oximod::{_error::oximod_error::OxiModError, Model};

impl Group {
    pub async fn get_group(group_id: String) -> Result<Group, OxiModError> {
        if let Some(group) = Group::find_one(doc! { "group_id": group_id }).await? {
            Ok(group)
        } else {
            Err(OxiModError::IndexError("Group not found".to_string()))
        }
    }

    pub async fn get_or_create_group(group_id: String) -> Result<Group, OxiModError> {
        if let Some(group) = Group::find_one(doc! { "group_id": &group_id }).await? {
            Ok(group)
        } else {
            let new_group = Group::new()
                .group_id(group_id)
                .convertable_currencies(vec![]);

            let _ = new_group.save().await?;
            Self::get_group(new_group.group_id).await
        }
    }

    #[allow(dead_code)]
    pub async fn add_group(group_id: String) -> Result<ObjectId, OxiModError> {
        let group = Group::new()
            .group_id(group_id)
            .convertable_currencies(vec![])
            .save()
            .await?;

        Ok(group)
    }
}

use digestible::Digestible;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema, Digestible)]
#[cfg_attr(feature = "sea-orm", derive(sea_orm::FromJsonQueryResult))]
pub enum Visibility {
    Public,
    Unlisted,
    /// Always visible to the owner.
    Private {
        visible_to: Vec<i64>,
    },
}
impl Default for Visibility {
    fn default() -> Self {
        Self::Public
    }
}

impl HasVisibility for (i64, Visibility) {
    fn visibility(&self) -> &Visibility {
        &self.1
    }

    fn is_owner(&self, user_id: i64) -> bool {
        self.0 == user_id
    }
}
pub trait HasVisibility {
    fn visibility(&self) -> &Visibility;

    fn is_owner(&self, user_id: i64) -> bool;

    fn requires_auth(&self) -> bool {
        match self.visibility() {
            Visibility::Public => false,
            Visibility::Unlisted => false,
            Visibility::Private { .. } => true,
        }
    }
    fn is_visible_to(&self, user_id: i64) -> bool {
        if self.is_owner(user_id) {
            return true;
        }
        match self.visibility() {
            Visibility::Public => true,
            Visibility::Unlisted => true,
            Visibility::Private { visible_to } => {
                if user_id == 0 {
                    return false;
                }
                visible_to.contains(&user_id)
            }
        }
    }
}

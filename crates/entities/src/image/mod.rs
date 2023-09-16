use digestible::Digestible;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

pub mod image;
pub mod post;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Digestible, ToSchema)]
#[serde(default)]
#[typeshare]
pub struct ImagePermissions {
    pub create: bool,
    /// Can view private images and delete images that are not owned by the user
    pub admin: bool,
    /// Can view public images
    pub view_public: bool,
}
impl Default for ImagePermissions {
    fn default() -> Self {
        Self {
            create: true,
            admin: false,
            view_public: true,
        }
    }
}

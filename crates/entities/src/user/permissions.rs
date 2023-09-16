use digestible::Digestible;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{image::ImagePermissions, paste::PastePermissions};

#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    FromJsonQueryResult,
    Digestible,
    ToSchema,
)]
#[serde(default)]
#[typeshare]
pub struct Permissions {
    pub image_permissions: ImagePermissions,
    pub paste_permissions: PastePermissions,
    pub user_permissions: UserPermissions,
    pub admin: bool,
}

impl Permissions {
    pub fn is_paste_admin(&self) -> bool {
        self.paste_permissions.admin || self.admin
    }
    pub fn is_image_admin(&self) -> bool {
        self.image_permissions.admin || self.admin
    }
    #[inline]
    pub fn new_admin() -> Self {
        Self {
            image_permissions: ImagePermissions {
                create: true,
                admin: true,
                view_public: true,
            },
            admin: true,
            paste_permissions: PastePermissions {
                create: true,
                admin: true,
                view_public: true,
            },
            user_permissions: UserPermissions {
                view_profile: true,
                edit_user: true,
                create_auth_token: true,
            },
        }
    }
    #[inline]
    pub fn new_anonymous() -> Self {
        Self {
            image_permissions: ImagePermissions {
                create: false,
                admin: false,
                view_public: true,
            },
            admin: true,
            paste_permissions: PastePermissions {
                create: false,
                admin: false,
                view_public: true,
            },
            user_permissions: UserPermissions {
                view_profile: true,
                edit_user: false,
                create_auth_token: false,
            },
        }
    }
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Digestible, ToSchema)]
#[serde(default)]
pub struct UserPermissions {
    pub edit_user: bool,
    pub view_profile: bool,
    pub create_auth_token: bool,
}
impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            view_profile: true,
            edit_user: false,
            create_auth_token: true,
        }
    }
}

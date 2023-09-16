use bytes::Bytes;
use digestible::Digestible;
use http::{
    header::{InvalidHeaderValue, CONTENT_LANGUAGE, CONTENT_TYPE},
    HeaderMap, HeaderName, HeaderValue,
};
use mime::{MediaType, JSON};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

#[inline(always)]
pub(crate) fn default_mime_type() -> String {
    "text/plain".to_string()
}
#[inline(always)]
pub(crate) fn default_charset() -> String {
    "UTF-8".to_string()
}
pub const PROGRAMMING_LANGUAGE: HeaderName = HeaderName::from_static("programming-language");

/// A file type
///
/// mime_type: The mime type of the file
/// charset: The charset of the file
/// programming_language: The programming language of the file
/// content_language: The content language of the file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema, Digestible)]
#[cfg_attr(feature = "sea-orm", derive(sea_orm::FromJsonQueryResult))]
#[serde(default)]
#[typeshare]
pub struct FileType {
    pub mime_type: String,
    pub charset: String,
    pub programming_language: Option<String>,
    pub content_language: Option<String>,
}
impl FileType {
    pub fn new_from_header(content_type: String) -> Self {
        let mime: MediaType = content_type.parse().unwrap();
        let charset = mime
            .param("charset")
            .map(|v| v.to_string())
            .unwrap_or(default_charset());
        let mime_type = mime.without_params().to_string();

        Self {
            mime_type,
            charset,
            programming_language: None,
            content_language: None,
        }
    }
    pub fn new(mime_type: String, charset: String) -> Self {
        Self {
            mime_type,
            charset,
            programming_language: None,
            content_language: None,
        }
    }
    /// Tries to fill in the missing information about the file type
    pub fn process_file(&mut self, file_name: impl AsRef<str>) {
        if self.mime_type == JSON {
            return;
        }
        if self.mime_type == "text/plain" && self.programming_language.is_none() {
            if let Some(lang) = file_name.as_ref().split('.').last() {
                if lang != "txt" {
                    // TODO guess programming language
                } else if lang == "json" {
                    self.mime_type = JSON.to_string();
                }
            }
        }
    }
    pub fn check(&self) -> bool {
        if HeaderValue::from_str(&self.mime_type_string()).is_err() {
            return false;
        }
        if let Some(lang) = &self.programming_language {
            if HeaderValue::from_str(lang).is_err() {
                return false;
            }
        }
        if let Some(lang) = &self.content_language {
            if HeaderValue::from_str(lang).is_err() {
                return false;
            }
        }
        true
    }
    pub fn as_media_type(&self) -> MediaType {
        self.mime_type_string().parse().unwrap()
    }
    pub fn mime_type_string(&self) -> String {
        let mut content_type = self.mime_type.clone();
        if self.charset != default_charset() {
            content_type.push_str("; charset=");
            content_type.push_str(&self.charset);
        }
        content_type
    }
    pub fn headers_owned(self) -> Result<HeaderMap, InvalidHeaderValue> {
        // Probably should not be allocating to a vec here.
        let mut headers = HeaderMap::with_capacity(3);
        let mime_type = HeaderValue::from_str(&self.mime_type_string())?;
        headers.insert(CONTENT_TYPE, mime_type);
        if let Some(lang) = self.programming_language.map(|v| Bytes::from(v)) {
            let lang = HeaderValue::from_maybe_shared(lang)?;
            headers.insert(PROGRAMMING_LANGUAGE, lang);
        }
        if let Some(lang) = self.content_language.map(|v| Bytes::from(v)) {
            let lang = HeaderValue::from_maybe_shared(lang)?;
            headers.insert(CONTENT_LANGUAGE, lang);
        }
        Ok(headers)
    }

    /// Nitro_Share will assume any data in the database is already valid.
    /// So this function is called.
    ///
    /// # Safety
    /// Header values need to be checked for validity before being used.
    /// Please call [`FileType::check`] before calling this function
    ///
    /// The code is not truly unsafe.
    /// However, an invalid header might not be supported by the client.
    pub fn headers_owned_unchecked(self) -> HeaderMap {
        // Probably should not be allocating to a vec here.
        let mut headers = HeaderMap::with_capacity(3);
        unsafe {
            let mime_type = HeaderValue::from_maybe_shared_unchecked(self.mime_type_string());
            headers.insert(CONTENT_TYPE, mime_type);
            if let Some(lang) = self.programming_language.map(|v| Bytes::from(v)) {
                let lang = HeaderValue::from_maybe_shared_unchecked(lang);
                headers.insert(PROGRAMMING_LANGUAGE, lang);
            }
            if let Some(lang) = self.content_language.map(|v| Bytes::from(v)) {
                let lang = HeaderValue::from_maybe_shared_unchecked(lang);
                headers.insert(CONTENT_LANGUAGE, lang);
            }
        }
        headers
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            mime_type: default_mime_type(),
            charset: default_charset(),
            programming_language: None,
            content_language: None,
        }
    }
}

//! MTProto adapter boundary only; no account connection, session, or Bot API exists.
use crate::model::MessageMeta;

pub struct MessageView<'a> {
    pub meta: &'a MessageMeta,
    pub text: &'a str,
    pub caption: Option<&'a str>,
    pub has_media: bool,
}

impl MessageView<'_> {
    pub fn effective_text(&self) -> &str {
        self.caption
            .filter(|caption| !caption.is_empty())
            .unwrap_or(self.text)
    }
}

use std::fmt::Formatter;

use crate::id::{
    marker::{EmojiMarker, TagMarker},
    Id,
};
use serde::{
    de::{Error, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_value::Value;

/// Emoji to use as the default way to react to a forum post.
///
/// Exactly one of `emoji_id` and `emoji_name` must be set.
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct DefaultReaction {
    /// ID of custom guild emoji.
    ///
    /// Conflicts with `emoji_name`.
    pub emoji_id: Option<Id<EmojiMarker>>,
    /// Unicode emoji character.
    ///
    /// Conflicts with `emoji_id`.
    pub emoji_name: Option<String>,
}

/// Tag that is able to be applied to a thread in a [`GuildForum`] [`Channel`].
///
/// May at most contain one of `emoji_id` and `emoji_name`.
///
/// [`Channel`]: super::Channel
/// [`GuildForum`]: super::ChannelType::GuildForum
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct ForumTag {
    /// ID of custom guild emoji.
    ///
    /// Conflicts with `emoji_name`.
    pub emoji_id: Option<Id<EmojiMarker>>,
    /// Unicode emoji character.
    ///
    /// Conflicts with `emoji_name`.
    pub emoji_name: Option<String>,
    /// ID of the tag.
    pub id: Id<TagMarker>,
    /// Whether the tag can only be added or removed by [`Member`]s with the
    /// [`MANAGE_THREADS`] permission.
    ///
    /// [`MANAGE_THREADS`]: crate::guild::Permissions::MANAGE_THREADS
    /// [`Member`]: crate::guild::Member
    pub moderated: bool,
    /// Name of the tag (0--20 characters).
    pub name: String,
}

impl<'de> Deserialize<'de> for ForumTag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            EmojiId,
            EmojiName,
            Id,
            Moderated,
            Name,
        }

        struct ForumTagVisitor;

        impl<'de> Visitor<'de> for ForumTagVisitor {
            type Value = ForumTag;

            fn expecting(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str("struct ForumTag")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<Self::Value, V::Error> {
                let mut emoji_id = None::<Id<_>>;
                let mut emoji_name = None::<Option<String>>;
                let mut id = None::<Id<TagMarker>>;
                let mut moderated = None::<bool>;
                let mut name = None::<String>;

                loop {
                    let key = match map.next_key() {
                        Ok(Some(key)) => key,
                        Ok(None) => break,
                        Err(why) => {
                            map.next_value::<IgnoredAny>()?;

                            tracing::trace!("ran into an unknown key: {why:?}");

                            continue;
                        }
                    };

                    match key {
                        Field::EmojiId => {
                            if emoji_id.is_some() {
                                return Err(Error::duplicate_field("emoji_id"));
                            }

                            let value: Value = map.next_value()?;

                            let possible_id = match value {
                                Value::U64(val) => Some(val),
                                Value::Option(Some(value)) => match *value {
                                    Value::Newtype(newtype) => {
                                        if let Value::String(string) = *newtype {
                                            Some(string.parse::<u64>().unwrap())
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                },
                                _ => None,
                            };

                            if let Some(id) = possible_id {
                                if id > 0 {
                                    emoji_id = Some(Id::new(id));
                                }
                            }
                        }
                        Field::EmojiName => {
                            if emoji_name.is_some() {
                                return Err(Error::duplicate_field("emoji_name"));
                            }

                            emoji_name = Some(map.next_value()?);
                        }
                        Field::Id => {
                            if id.is_some() {
                                return Err(Error::duplicate_field("id"));
                            }

                            id = Some(map.next_value()?);
                        }
                        Field::Moderated => {
                            if moderated.is_some() {
                                return Err(Error::duplicate_field("moderated"));
                            }

                            moderated = Some(map.next_value()?);
                        }
                        Field::Name => {
                            if name.is_some() {
                                return Err(Error::duplicate_field("name"));
                            }

                            name = Some(map.next_value()?);
                        }
                    }
                }

                Ok(ForumTag {
                    emoji_id,
                    emoji_name: emoji_name.unwrap_or_default(),
                    id: id.ok_or_else(|| Error::missing_field("id"))?,
                    moderated: moderated.ok_or_else(|| Error::missing_field("moderated"))?,
                    name: name.ok_or_else(|| Error::missing_field("name"))?,
                })
            }
        }

        deserializer.deserialize_struct(
            "ForumTag",
            &["emoji_id", "emoji_name", "id", "moderated", "name"],
            ForumTagVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{DefaultReaction, ForumTag};
    use crate::id::{
        marker::{EmojiMarker, TagMarker},
        Id,
    };
    use serde_test::Token;

    const EMOJI_ID: Id<EmojiMarker> = Id::new(1);
    const TAG_ID: Id<TagMarker> = Id::new(2);

    #[test]
    fn default_reaction() {
        let value = DefaultReaction {
            emoji_id: None,
            emoji_name: Some("name".to_owned()),
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "DefaultReaction",
                    len: 2,
                },
                Token::Str("emoji_id"),
                Token::None,
                Token::Str("emoji_name"),
                Token::Some,
                Token::Str("name"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn forum_tag() {
        let value = ForumTag {
            emoji_id: Some(EMOJI_ID),
            emoji_name: None,
            id: TAG_ID,
            moderated: false,
            name: "other".into(),
        };

        serde_test::assert_de_tokens(
            &value,
            &[
                Token::Struct {
                    name: "ForumTag",
                    len: 5,
                },
                Token::Str("emoji_id"),
                Token::Some,
                Token::NewtypeStruct { name: "Id" },
                Token::Str("1"),
                Token::Str("emoji_name"),
                Token::None,
                Token::Str("id"),
                Token::NewtypeStruct { name: "Id" },
                Token::Str("2"),
                Token::Str("moderated"),
                Token::Bool(false),
                Token::Str("name"),
                Token::Str("other"),
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn forum_tag_emoji_id_zero() {
        let deserialized = serde_json::from_str::<ForumTag>("{\n   \"name\":\"other\",\n   \"moderated\":false,\n   \"id\":\"2\",\n   \"emoji_name\":\"emoji_name\",\n   \"emoji_id\":0\n}");

        assert!(deserialized.is_ok());

        let tag = deserialized.unwrap();

        assert!(tag.emoji_id.is_none());
        assert!(tag.emoji_name.is_some());
    }
}

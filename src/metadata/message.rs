use saphyr::{MarkedYaml, YamlData};

use super::{
    error::{
        ExpectedType, MetadataParsingErrorReason, MultilingualMessageContentsError,
        ParseMetadataError,
    },
    yaml::{
        YamlObjectType, as_string_node, get_as_hash, get_required_string_value,
        get_strings_vec_value, parse_condition,
    },
};

/// Codes used to indicate the type of a message.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MessageType {
    /// A notification message that is of no significant severity.
    #[default]
    Say,
    /// A warning message, used to indicate that an issue may be present that
    /// the user may wish to act on.
    Warn,
    /// An error message, used to indicate that an issue that requires user
    /// action is present.
    Error,
}

/// Represents a message's localised text content.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageContent {
    text: String,
    language: String,
}

impl MessageContent {
    /// The code for the default language assumed for message content.
    pub const DEFAULT_LANGUAGE: &'static str = "en";

    /// Construct a [MessageContent] object with the given text in the default
    /// language.
    #[must_use]
    pub fn new(text: String) -> Self {
        MessageContent {
            text,
            ..Default::default()
        }
    }

    /// Set the language code to the given value.
    #[must_use]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// Get the message text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the text's language code.
    pub fn language(&self) -> &str {
        &self.language
    }
}

impl std::default::Default for MessageContent {
    /// Construct a [MessageContent] object with an empty message string and the
    /// default language.
    #[must_use]
    fn default() -> Self {
        Self {
            text: Default::default(),
            language: MessageContent::DEFAULT_LANGUAGE.into(),
        }
    }
}

/// Choose a [MessageContent] object from those given in `content` based on the
/// given `language`.
///
/// The locale or language code for the preferred language to select. Codes are
/// of the form `[language code]_[country code]`.
///
/// * If the vector only contains a single element, that element is returned.
/// * If content with a language that exactly matches the given locale or
///   language code is present, that content is returned.
/// * If a locale code is given and there is no exact match but content for that
///   locale's language is present, that content is returned.
/// * If a language code is given and there is no exact match but content for a
///   locale in that language is present, that content is returned.
/// * If no locale or language code matches are found and content in the default
///   language is present, that content is returned.
/// * Otherwise, an empty [Option] is returned.
pub fn select_message_content<'a>(
    content: &'a [MessageContent],
    language: &str,
) -> Option<&'a MessageContent> {
    if content.is_empty() {
        None
    } else if let [c] = content {
        Some(c)
    } else {
        let language_code = language.split_once('_').map(|p| p.0);

        let mut matched = None;
        let mut english = None;

        for mc in content {
            if mc.language == language {
                return Some(mc);
            } else if matched.is_none() {
                if language_code.is_some_and(|c| c == mc.language) {
                    matched = Some(mc);
                } else if language_code.is_none() {
                    if let Some((content_language_code, _)) = mc.language.split_once('_') {
                        if content_language_code == language {
                            matched = Some(mc);
                        }
                    }
                }

                if mc.language == MessageContent::DEFAULT_LANGUAGE {
                    english = Some(mc);
                }
            }
        }

        if matched.is_some() {
            matched
        } else if english.is_some() {
            english
        } else {
            None
        }
    }
}

/// Represents a message with localisable text content.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Message {
    message_type: MessageType,
    content: Vec<MessageContent>,
    condition: Option<String>,
}

impl Message {
    /// Construct a [Message] with the given type and a content string in the
    /// language given by [MessageContent::DEFAULT_LANGUAGE].
    #[must_use]
    pub fn new(message_type: MessageType, content: String) -> Self {
        Self {
            message_type,
            content: vec![MessageContent {
                text: content,
                language: MessageContent::DEFAULT_LANGUAGE.to_string(),
            }],
            condition: None,
        }
    }

    /// Construct a [Message] with the given type and content. If more than one
    /// [MessageContent] object is given, one must use
    /// the language code given by [MessageContent::DEFAULT_LANGUAGE].
    pub fn multilingual(
        message_type: MessageType,
        content: Vec<MessageContent>,
    ) -> Result<Self, MultilingualMessageContentsError> {
        validate_message_contents(&content)?;

        Ok(Self {
            message_type,
            content,
            condition: None,
        })
    }

    /// Set the condition string.
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Get the message type.
    pub fn message_type(&self) -> MessageType {
        self.message_type
    }

    /// Get the message content.
    pub fn content(&self) -> &[MessageContent] {
        &self.content
    }

    /// Get the condition string.
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

pub(crate) fn validate_message_contents(
    contents: &[MessageContent],
) -> Result<(), MultilingualMessageContentsError> {
    if contents.len() > 1 {
        let english_string_exists = contents
            .iter()
            .any(|c| c.language == MessageContent::DEFAULT_LANGUAGE);

        if !english_string_exists {
            return Err(MultilingualMessageContentsError {});
        }
    }

    Ok(())
}

impl TryFrom<&MarkedYaml> for MessageContent {
    type Error = ParseMetadataError;

    fn try_from(value: &MarkedYaml) -> Result<Self, Self::Error> {
        let hash = get_as_hash(value, YamlObjectType::MessageContent)?;

        let text =
            get_required_string_value(value.span.start, hash, "text", YamlObjectType::Message)?;

        let language =
            get_required_string_value(value.span.start, hash, "lang", YamlObjectType::Message)?;

        Ok(MessageContent {
            text: text.to_string(),
            language: language.to_string(),
        })
    }
}

pub(crate) fn parse_message_contents_yaml(
    value: &MarkedYaml,
    key: &'static str,
    parent_yaml_type: YamlObjectType,
) -> Result<Vec<MessageContent>, ParseMetadataError> {
    let contents = match &value.data {
        YamlData::String(s) => {
            vec![MessageContent {
                text: s.to_string(),
                language: MessageContent::DEFAULT_LANGUAGE.to_string(),
            }]
        }
        YamlData::Array(a) => a
            .iter()
            .map(MessageContent::try_from)
            .collect::<Result<Vec<MessageContent>, _>>()?,
        _ => {
            return Err(ParseMetadataError::unexpected_value_type(
                value.span.start,
                key,
                parent_yaml_type,
                ExpectedType::ArrayOrString,
            ));
        }
    };

    if validate_message_contents(&contents).is_err() {
        Err(ParseMetadataError::new(
            value.span.start,
            MetadataParsingErrorReason::InvalidMultilingualMessageContents,
        ))
    } else {
        Ok(contents)
    }
}

impl TryFrom<&MarkedYaml> for Message {
    type Error = ParseMetadataError;

    fn try_from(value: &MarkedYaml) -> Result<Self, Self::Error> {
        let hash = get_as_hash(value, YamlObjectType::Message)?;

        let message_type =
            get_required_string_value(value.span.start, hash, "type", YamlObjectType::Message)?;
        let message_type = match message_type {
            "warn" => MessageType::Warn,
            "error" => MessageType::Error,
            _ => MessageType::Say,
        };

        let mut content = match hash.get(&as_string_node("content")) {
            Some(n) => parse_message_contents_yaml(n, "content", YamlObjectType::Message)?,
            None => {
                return Err(ParseMetadataError::missing_key(
                    value.span.start,
                    "content",
                    YamlObjectType::Message,
                ));
            }
        };

        let subs = get_strings_vec_value(hash, "subs", YamlObjectType::Message)?;

        if !subs.is_empty() {
            for mc in &mut content {
                for (index, sub) in subs.iter().enumerate() {
                    let placeholder = format!("{}", index);
                    if !mc.text.contains(&placeholder) {
                        return Err(ParseMetadataError::new(
                            value.span.start,
                            MetadataParsingErrorReason::MissingPlaceholder(sub.to_string(), index),
                        ));
                    }

                    mc.text = mc.text.replace(&placeholder, sub);
                }
            }
        }

        let condition = parse_condition(hash, YamlObjectType::Message)?;

        Ok(Message {
            message_type,
            content,
            condition,
        })
    }
}

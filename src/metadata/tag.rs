use std::str::FromStr;

use loot_condition_interpreter::Expression;
use saphyr::YamlData;

use super::yaml::{YamlObjectType, get_required_string_value, get_string_value};
use crate::error::{GeneralError, YamlParseError};

/// Represents whether a Bash Tag suggestion is for addition or removal.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagSuggestion {
    #[default]
    Addition,
    Removal,
}

/// Represents a Bash Tag suggestion for a plugin.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Tag {
    name: String,
    suggestion: TagSuggestion,
    condition: Option<String>,
}

impl Tag {
    /// Create a [Tag] suggestion for the given tag name.
    #[must_use]
    pub fn new(name: String, suggestion: TagSuggestion) -> Self {
        Self {
            name,
            suggestion,
            condition: None,
        }
    }

    /// Set the condition string.
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Get the tag's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get if the tag should be added.
    pub fn is_addition(&self) -> bool {
        self.suggestion == TagSuggestion::Addition
    }

    /// Get the condition string.
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}

impl TryFrom<&saphyr::MarkedYaml> for Tag {
    type Error = GeneralError;

    fn try_from(value: &saphyr::MarkedYaml) -> Result<Self, Self::Error> {
        match &value.data {
            YamlData::String(s) => {
                let (name, suggestion) = name_and_suggestion(s);
                Ok(Tag {
                    name,
                    suggestion,
                    condition: None,
                })
            }
            YamlData::Hash(h) => {
                let name =
                    get_required_string_value(value.span.start, h, "name", YamlObjectType::Tag)?;

                let condition = match get_string_value(h, "condition", YamlObjectType::Tag)? {
                    Some(n) => {
                        Expression::from_str(n)?;
                        Some(n.to_string())
                    }
                    None => None,
                };

                let (name, suggestion) = name_and_suggestion(name);
                Ok(Tag {
                    name,
                    suggestion,
                    condition,
                })
            }
            _ => Err(YamlParseError::new(
                value.span.start,
                "'tag' object must be a map or string".into(),
            )
            .into()),
        }
    }
}

fn name_and_suggestion(value: &str) -> (String, TagSuggestion) {
    if let Some(name) = value.strip_prefix("-") {
        (name.to_string(), TagSuggestion::Removal)
    } else {
        (value.to_string(), TagSuggestion::Addition)
    }
}

use saphyr::{MarkedYaml, YamlData};

use super::error::ExpectedType;
use super::error::ParseMetadataError;

use super::yaml::{YamlObjectType, get_required_string_value};
use super::yaml_emit::EmitYaml;
use super::yaml_emit::YamlEmitter;

/// Represents a URL at which the parent plugin can be found.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Location {
    url: String,
    name: Option<String>,
}

impl Location {
    /// Construct a [Location] with the given URL.
    #[must_use]
    pub fn new(url: String) -> Self {
        Location {
            url,
            ..Default::default()
        }
    }

    /// Set a name for the URL, eg. the page or site name.
    #[must_use]
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Get the URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the descriptive name of this location.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl TryFrom<&MarkedYaml> for Location {
    type Error = ParseMetadataError;

    fn try_from(value: &MarkedYaml) -> Result<Self, Self::Error> {
        match &value.data {
            YamlData::String(s) => Ok(Location {
                url: s.clone(),
                name: None,
            }),
            YamlData::Hash(h) => {
                let link = get_required_string_value(
                    value.span.start,
                    h,
                    "link",
                    YamlObjectType::Location,
                )?;
                let name = get_required_string_value(
                    value.span.start,
                    h,
                    "name",
                    YamlObjectType::Location,
                )?;

                Ok(Location {
                    url: link.to_string(),
                    name: Some(name.to_string()),
                })
            }
            _ => Err(ParseMetadataError::unexpected_type(
                value.span.start,
                YamlObjectType::Location,
                ExpectedType::MapOrString,
            )),
        }
    }
}

impl EmitYaml for Location {
    fn is_scalar(&self) -> bool {
        self.name.is_none()
    }

    fn emit_yaml(&self, emitter: &mut YamlEmitter) {
        if let Some(name) = &self.name {
            emitter.begin_map();

            emitter.map_key("link");
            emitter.single_quoted_str(&self.url);

            emitter.map_key("name");
            emitter.single_quoted_str(name);

            emitter.end_map();
        } else {
            emitter.single_quoted_str(&self.url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod emit_yaml {
        use crate::metadata::emit;

        use super::*;

        #[test]
        fn should_emit_url_only_if_there_is_no_name() {
            let location = Location::new("http://www.example.com".into());
            let yaml = emit(&location);

            assert_eq!(format!("'{}'", location.url), yaml);
        }

        #[test]
        fn should_emit_map_if_there_is_a_name() {
            let location =
                Location::new("http://www.example.com".into()).with_name("example".into());
            let yaml = emit(&location);

            assert_eq!(
                format!(
                    "link: '{}'\nname: '{}'",
                    location.url,
                    location.name.unwrap()
                ),
                yaml
            );
        }
    }
}

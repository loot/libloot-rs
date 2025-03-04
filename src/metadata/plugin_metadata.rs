use fancy_regex::Regex;
use saphyr::MarkedYaml;

use crate::regex;

use super::{
    error::{MetadataParsingErrorReason, ParseMetadataError, RegexError},
    file::File,
    location::Location,
    message::Message,
    plugin_cleaning_data::PluginCleaningData,
    tag::Tag,
    yaml::{
        YamlObjectType, get_as_hash, get_as_slice, get_required_string_value, get_string_value,
    },
};

pub(crate) const GHOST_FILE_EXTENSION: &str = ".ghost";

/// Represents a plugin's metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PluginMetadata {
    name: PluginName,
    group: Option<String>,
    load_after: Vec<File>,
    requirements: Vec<File>,
    incompatibilities: Vec<File>,
    messages: Vec<Message>,
    tags: Vec<Tag>,
    dirty_info: Vec<PluginCleaningData>,
    clean_info: Vec<PluginCleaningData>,
    locations: Vec<Location>,
}

impl PluginMetadata {
    /// Construct a [PluginMetadata] object with no metadata for a plugin with
    /// the given filename.
    pub fn new(name: &str) -> Result<Self, RegexError> {
        Ok(Self {
            name: PluginName::new(name)?,
            ..Default::default()
        })
    }

    /// Get the plugin name.
    pub fn name(&self) -> &str {
        &self.name.string
    }

    /// Get the plugin's group.
    ///
    /// The [Option] is `None` if no group is explicitly set.
    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    /// Get the plugins that the plugin must load after.
    pub fn load_after_files(&self) -> &[File] {
        &self.load_after
    }

    /// Get the files that the plugin requires to be installed.
    pub fn requirements(&self) -> &[File] {
        &self.requirements
    }

    /// Get the files that the plugin is incompatible with.
    pub fn incompatibilities(&self) -> &[File] {
        &self.incompatibilities
    }

    /// Get the plugin's messages.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get the plugin's Bash Tag suggestions.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Get the plugin's dirty plugin information.
    pub fn dirty_info(&self) -> &[PluginCleaningData] {
        &self.dirty_info
    }

    /// Get the plugin's clean plugin information.
    pub fn clean_info(&self) -> &[PluginCleaningData] {
        &self.clean_info
    }

    /// Get the locations at which this plugin can be found.
    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    /// Set the plugin's group.
    pub fn set_group(&mut self, group: &str) {
        self.group = Some(group.to_string())
    }

    /// Unsets the plugin's group, so that it is implicitly a member of the
    /// default group.
    pub fn unset_group(&mut self) {
        self.group = None
    }

    /// Get the plugins that the plugin must load after.
    pub fn set_load_after_files(&mut self, files: Vec<File>) {
        self.load_after = files;
    }

    /// Get the files that the plugin requires to be installed.
    pub fn set_requirements(&mut self, files: Vec<File>) {
        self.requirements = files;
    }

    /// Get the files that the plugin is incompatible with.
    pub fn set_incompatibilities(&mut self, files: Vec<File>) {
        self.incompatibilities = files;
    }

    /// Get the plugin's messages.
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
    }

    /// Get the plugin's Bash Tag suggestions.
    pub fn set_tags(&mut self, tags: Vec<Tag>) {
        self.tags = tags;
    }

    /// Get the plugin's dirty plugin information.
    pub fn set_dirty_info(&mut self, dirty_info: Vec<PluginCleaningData>) {
        self.dirty_info = dirty_info;
    }

    /// Get the plugin's clean plugin information.
    pub fn set_clean_info(&mut self, clean_info: Vec<PluginCleaningData>) {
        self.clean_info = clean_info;
    }

    /// Get the locations at which this plugin can be found.
    pub fn set_locations(&mut self, locations: Vec<Location>) {
        self.locations = locations;
    }

    /// Merge metadata from the given [PluginMetadata] object into this object.
    ///
    /// If an equal metadata object already exists in this PluginMetadata
    /// object, it is not duplicated. This object's group is replaced by the
    /// given object's group if the latter is explicit.
    pub fn merge_metadata(&mut self, plugin: &PluginMetadata) {
        if plugin.has_name_only() {
            return;
        }

        if self.group.is_none() && plugin.group.is_some() {
            self.group = plugin.group.clone();
        }

        merge_vecs(&mut self.load_after, &plugin.load_after);
        merge_vecs(&mut self.requirements, &plugin.requirements);
        merge_vecs(&mut self.incompatibilities, &plugin.incompatibilities);
        merge_vecs(&mut self.tags, &plugin.tags);
        self.messages.extend(plugin.messages.iter().cloned());
        merge_vecs(&mut self.dirty_info, &plugin.dirty_info);
        merge_vecs(&mut self.clean_info, &plugin.clean_info);
        merge_vecs(&mut self.locations, &plugin.locations);
    }

    /// Check if no plugin metadata is set.
    pub fn has_name_only(&self) -> bool {
        self.group.is_none()
            && self.load_after.is_empty()
            && self.requirements.is_empty()
            && self.incompatibilities.is_empty()
            && self.messages.is_empty()
            && self.tags.is_empty()
            && self.dirty_info.is_empty()
            && self.clean_info.is_empty()
            && self.locations.is_empty()
    }

    /// Check if the plugin name is a regular expression.
    ///
    /// Returns `true` if the plugin name contains any of the characters `:\*?|`
    /// and `false` otherwise.
    pub fn is_regex_plugin(&self) -> bool {
        self.name.regex.is_some()
    }

    /// Check if the given plugin name matches this plugin metadata object's
    /// name field.
    ///
    /// If the name field is a regular expression, the given plugin name will be
    /// matched against it, otherwise the strings will be compared
    /// case-insensitively. The given plugin name must be literal, i.e. not a
    /// regular expression.
    pub fn name_matches(&self, other_name: &str) -> bool {
        if let Some(regex) = &self.name.regex {
            regex.is_match(other_name).inspect_err(|e| {
                log::error!("Encountered an error while trying to match the regex {} to the string {}: {}", regex.as_str(), other_name, e);
            }).unwrap_or(false)
        } else {
            unicase::eq(self.name.string.as_str(), other_name)
        }
    }

    /// Serialises the plugin metadata as YAML.
    pub fn as_yaml(&self) -> String {
        todo!()
    }
}

#[derive(Clone, Debug, Default)]
struct PluginName {
    string: String,
    regex: Option<Regex>,
}

impl PluginName {
    fn new(name: &str) -> Result<Self, Box<fancy_regex::Error>> {
        let name = trim_dot_ghost(name).to_string();

        if is_regex_name(&name) {
            let regex = regex(&format!("^{}$", &name))?;
            Ok(Self {
                string: name,
                regex: Some(regex),
            })
        } else {
            Ok(Self {
                string: name,
                regex: None,
            })
        }
    }
}

impl std::cmp::PartialEq for PluginName {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl std::cmp::Eq for PluginName {}

impl std::cmp::PartialOrd for PluginName {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for PluginName {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.string.cmp(&other.string)
    }
}

impl std::hash::Hash for PluginName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string.hash(state);
    }
}

pub(crate) fn trim_dot_ghost(string: &str) -> &str {
    if iends_with_ascii(string, GHOST_FILE_EXTENSION) {
        &string[..(string.len() - 6)]
    } else {
        string
    }
}

pub(crate) fn iends_with_ascii(string: &str, suffix: &str) -> bool {
    // as_bytes().into_iter() is faster than bytes().
    string.len() >= suffix.len()
        && string
            .as_bytes()
            .iter()
            .rev()
            .zip(suffix.as_bytes().iter().rev())
            .all(|(string_byte, suffix_byte)| string_byte.eq_ignore_ascii_case(suffix_byte))
}

fn is_regex_name(name: &str) -> bool {
    name.contains(|c| ":\\*?|".chars().any(|n| c == n))
}

fn merge_vecs<T: Clone + PartialEq>(target: &mut Vec<T>, source: &[T]) {
    let initial_target_len = target.len();
    for element in source {
        if !target[..initial_target_len].contains(element) {
            target.push(element.clone())
        }
    }
}

impl TryFrom<&MarkedYaml> for PluginMetadata {
    type Error = ParseMetadataError;

    fn try_from(value: &MarkedYaml) -> Result<Self, Self::Error> {
        let hash = get_as_hash(value, YamlObjectType::PluginMetadata)?;

        let name = get_required_string_value(
            value.span.start,
            hash,
            "name",
            YamlObjectType::PluginMetadata,
        )?;
        let name = match PluginName::new(name) {
            Ok(n) => n,
            Err(e) => {
                return Err(ParseMetadataError::new(
                    value.span.start,
                    MetadataParsingErrorReason::InvalidRegex(e),
                ));
            }
        };

        let group = get_string_value(hash, "group", YamlObjectType::PluginMetadata)?;

        let load_after = get_vec::<File>(hash, "after")?;
        let requirements = get_vec::<File>(hash, "req")?;
        let incompatibilities = get_vec::<File>(hash, "inc")?;
        let messages = get_vec::<Message>(hash, "msg")?;
        let tags = get_vec::<Tag>(hash, "tag")?;
        let dirty_info = get_vec::<PluginCleaningData>(hash, "dirty")?;
        let clean_info = get_vec::<PluginCleaningData>(hash, "clean")?;
        let locations = get_vec::<Location>(hash, "url")?;

        Ok(PluginMetadata {
            name,
            group: group.map(|g| g.1.to_string()),
            load_after,
            requirements,
            incompatibilities,
            messages,
            dirty_info,
            clean_info,
            tags,
            locations,
        })
    }
}

fn get_vec<'a, T: TryFrom<&'a MarkedYaml, Error = impl Into<ParseMetadataError>>>(
    hash: &'a saphyr::AnnotatedHash<MarkedYaml>,
    key: &'static str,
) -> Result<Vec<T>, ParseMetadataError> {
    get_as_slice(hash, key, YamlObjectType::PluginMetadata)?
        .iter()
        .map(|e| T::try_from(e).map_err(Into::into))
        .collect::<Result<Vec<T>, _>>()
}

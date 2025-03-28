#include "api/convert.h"

#include "api/exception.h"

namespace loot {
// To public types
/////////////////////

std::string convert(const ::rust::String& string) {
  return std::string(string);
}

loot::Group convert(const loot::rust::Group& group) {
  return loot::Group(std::string(group.name()),
                     convert<std::string>(group.after_groups()),
                     std::string(group.description()));
}

loot::File convert(const loot::rust::File& file) {
  return loot::File(std::string(file.filename().as_str()),
                    std::string(file.display_name()),
                    std::string(file.condition()),
                    convert<loot::MessageContent>(file.detail()));
}

loot::MessageType convert(loot::rust::MessageType messageType) {
  switch (messageType) {
    case loot::rust::MessageType::say:
      return loot::MessageType::say;
    case loot::rust::MessageType::warn:
      return loot::MessageType::warn;
    case loot::rust::MessageType::error:
      return loot::MessageType::error;
    default:
      throw std::logic_error("Unsupported MessageType value");
  }
}

loot::MessageContent convert(const loot::rust::MessageContent& content) {
  return loot::MessageContent(std::string(content.text()),
                              std::string(content.language()));
}

loot::Message convert(const loot::rust::Message& message) {
  return loot::Message(convert(message.message_type()),
                       convert<loot::MessageContent>(message.content()),
                       std::string(message.condition()));
}

loot::Tag convert(const loot::rust::Tag& tag) {
  return loot::Tag(
      std::string(tag.name()), tag.is_addition(), std::string(tag.condition()));
}

loot::PluginCleaningData convert(const loot::rust::PluginCleaningData& data) {
  return loot::PluginCleaningData(data.crc(),
                                  std::string(data.cleaning_utility()),
                                  convert<loot::MessageContent>(data.detail()),
                                  data.itm_count(),
                                  data.deleted_reference_count(),
                                  data.deleted_navmesh_count());
}

loot::Location convert(const loot::rust::Location& location) {
  return loot::Location(std::string(location.url()),
                        std::string(location.name()));
}

loot::PluginMetadata convert(const loot::rust::PluginMetadata& metadata) {
  auto output = loot::PluginMetadata(std::string(metadata.name()));

  if (!metadata.group().empty()) {
    output.SetGroup(std::string(metadata.group()));
  }

  output.SetLoadAfterFiles(convert<loot::File>(metadata.load_after_files()));
  output.SetRequirements(convert<loot::File>(metadata.requirements()));
  output.SetIncompatibilities(
      convert<loot::File>(metadata.incompatibilities()));
  output.SetMessages(convert<loot::Message>(metadata.messages()));
  output.SetTags(convert<loot::Tag>(metadata.tags()));
  output.SetDirtyInfo(convert<loot::PluginCleaningData>(metadata.dirty_info()));
  output.SetCleanInfo(convert<loot::PluginCleaningData>(metadata.clean_info()));
  output.SetLocations(convert<loot::Location>(metadata.locations()));

  return output;
}

std::optional<loot::EdgeType> convert(uint8_t edgeType) {
  switch (edgeType) {
    case static_cast<uint8_t>(loot::rust::EdgeType::hardcoded):
      return loot::EdgeType::hardcoded;
    case static_cast<uint8_t>(loot::rust::EdgeType::masterFlag):
      return loot::EdgeType::masterFlag;
    case static_cast<uint8_t>(loot::rust::EdgeType::master):
      return loot::EdgeType::master;
    case static_cast<uint8_t>(loot::rust::EdgeType::masterlistRequirement):
      return loot::EdgeType::masterlistRequirement;
    case static_cast<uint8_t>(loot::rust::EdgeType::userRequirement):
      return loot::EdgeType::userRequirement;
    case static_cast<uint8_t>(loot::rust::EdgeType::masterlistLoadAfter):
      return loot::EdgeType::masterlistLoadAfter;
    case static_cast<uint8_t>(loot::rust::EdgeType::userLoadAfter):
      return loot::EdgeType::userLoadAfter;
    case static_cast<uint8_t>(loot::rust::EdgeType::masterlistGroup):
      return loot::EdgeType::masterlistGroup;
    case static_cast<uint8_t>(loot::rust::EdgeType::userGroup):
      return loot::EdgeType::userGroup;
    case static_cast<uint8_t>(loot::rust::EdgeType::recordOverlap):
      return loot::EdgeType::recordOverlap;
    case static_cast<uint8_t>(loot::rust::EdgeType::assetOverlap):
      return loot::EdgeType::assetOverlap;
    case static_cast<uint8_t>(loot::rust::EdgeType::tieBreak):
      return loot::EdgeType::tieBreak;
    case static_cast<uint8_t>(loot::rust::EdgeType::blueprintMaster):
      return loot::EdgeType::blueprintMaster;
    default:
      return std::nullopt;
  }
}

loot::Vertex convert(const loot::rust::Vertex& vertex) {
  try {
    const auto outEdgeType = convert(vertex.out_edge_type());
    if (outEdgeType.has_value()) {
      return loot::Vertex(std::string(vertex.name()), outEdgeType.value());
    } else {
      return loot::Vertex(std::string(vertex.name()));
    }
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }
}

// From public types
///////////////////////

::rust::Box<loot::rust::Group> convert(const loot::Group& group) {
  auto output = loot::rust::new_group(group.GetName());
  output->set_after_groups(convert(group.GetAfterGroups()));
  output->set_description(group.GetDescription());

  return output;
}

::rust::Box<loot::rust::File> convert(const loot::File& file) {
  auto output = loot::rust::new_file(std::string(file.GetName()));
  output->set_display_name(file.GetDisplayName());

  try {
    output->set_detail(
        ::rust::Slice(convert<loot::rust::MessageContent>(file.GetDetail())));
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }

  output->set_condition(file.GetCondition());

  return output;
}

loot::rust::MessageType convert(loot::MessageType messageType) {
  switch (messageType) {
    case loot::MessageType::say:
      return loot::rust::MessageType::say;
    case loot::MessageType::warn:
      return loot::rust::MessageType::warn;
    case loot::MessageType::error:
      return loot::rust::MessageType::error;
    default:
      throw std::logic_error("Unsupported MessageType value");
  }
}

::rust::Box<loot::rust::MessageContent> convert(
    const loot::MessageContent& content) {
  auto output = loot::rust::new_message_content(content.GetText());
  output->set_language(content.GetLanguage());

  return output;
}

::rust::Box<loot::rust::Message> convert(const loot::Message& message) {
  try {
    auto output = loot::rust::multilingual_message(
        convert(message.GetType()),
        ::rust::Slice(
            convert<loot::rust::MessageContent>(message.GetContent())));
    output->set_condition(message.GetCondition());

    return output;
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }
}

::rust::Box<loot::rust::Tag> convert(const loot::Tag& tag) {
  try {
    const auto suggestion = tag.IsAddition()
                                ? loot::rust::TagSuggestion::Addition
                                : loot::rust::TagSuggestion::Removal;
    auto output = loot::rust::new_tag(tag.GetName(), suggestion);
    output->set_condition(tag.GetCondition());

    return output;
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }
}

::rust::Box<loot::rust::PluginCleaningData> convert(
    const loot::PluginCleaningData& data) {
  auto output = loot::rust::new_plugin_cleaning_data(data.GetCRC(),
                                                     data.GetCleaningUtility());
  try {
    output->set_detail(
        ::rust::Slice(convert<loot::rust::MessageContent>(data.GetDetail())));
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }

  output->set_itm_count(data.GetITMCount());
  output->set_deleted_reference_count(data.GetDeletedReferenceCount());
  output->set_deleted_navmesh_count(data.GetDeletedNavmeshCount());

  return output;
}

::rust::Box<loot::rust::Location> convert(const loot::Location& location) {
  auto output = loot::rust::new_location(location.GetURL());
  output->set_name(location.GetName());

  return output;
}

::rust::Box<loot::rust::PluginMetadata> convert(
    const loot::PluginMetadata& metadata) {
  try {
    auto output = loot::rust::new_plugin_metadata(metadata.GetName());

    if (metadata.GetGroup().has_value()) {
      output->set_group(metadata.GetGroup().value());
    }

    output->set_load_after_files(
        ::rust::Slice(convert<loot::rust::File>(metadata.GetLoadAfterFiles())));
    output->set_requirements(
        ::rust::Slice(convert<loot::rust::File>(metadata.GetRequirements())));
    output->set_incompatibilities(::rust::Slice(
        convert<loot::rust::File>(metadata.GetIncompatibilities())));
    output->set_messages(
        ::rust::Slice(convert<loot::rust::Message>(metadata.GetMessages())));
    output->set_tags(
        ::rust::Slice(convert<loot::rust::Tag>(metadata.GetTags())));
    output->set_dirty_info(::rust::Slice(
        convert<loot::rust::PluginCleaningData>(metadata.GetDirtyInfo())));
    output->set_clean_info(::rust::Slice(
        convert<loot::rust::PluginCleaningData>(metadata.GetCleanInfo())));
    output->set_locations(
        ::rust::Slice(convert<loot::rust::Location>(metadata.GetLocations())));

    return output;
  } catch (const ::rust::Error& e) {
    std::rethrow_exception(mapError(e));
  }
}

// Between containers
////////////////////////

::rust::Vec<::rust::String> convert(const std::vector<std::string>& vector) {
  ::rust::Vec<::rust::String> strings;
  for (const auto& str : vector) {
    strings.push_back(str);
  }

  return strings;
}
}

#ifndef LOOT_API_GAME
#define LOOT_API_GAME

#include <map>

#include "api/database.h"
#include "api/plugin.h"
#include "libloot-cxx/src/lib.rs.h"
#include "loot/game_interface.h"
#include "loot/metadata/filename.h"
#include "rust/cxx.h"

namespace loot {
class Game final : public GameInterface {
public:
  explicit Game(const GameType gameType,
                const std::filesystem::path& gamePath,
                const std::filesystem::path& gameLocalDataPath = "");

  // Game Interface Methods //
  ////////////////////////////

  GameType GetType() const override;

  std::vector<std::filesystem::path> GetAdditionalDataPaths() const;

  void SetAdditionalDataPaths(
      const std::vector<std::filesystem::path>& additionalDataPaths) override;

  DatabaseInterface& GetDatabase() override;
  const DatabaseInterface& GetDatabase() const override;

  bool IsValidPlugin(const std::filesystem::path& pluginPath) const override;

  void LoadPlugins(const std::vector<std::filesystem::path>& pluginPaths,
                   bool loadHeadersOnly) override;

  void ClearLoadedPlugins() override;

  const PluginInterface* GetPlugin(
      const std::string& pluginName) const override;

  std::vector<const PluginInterface*> GetLoadedPlugins() const override;

  std::vector<std::string> SortPlugins(
      const std::vector<std::string>& pluginFilenames) override;

  void LoadCurrentLoadOrderState() override;

  bool IsLoadOrderAmbiguous() const override;

  std::filesystem::path GetActivePluginsFilePath() const override;

  bool IsPluginActive(const std::string& pluginName) const override;

  std::vector<std::string> GetLoadOrder() const override;

  void SetLoadOrder(const std::vector<std::string>& loadOrder) override;

private:
  ::rust::Box<loot::rust::Game> game_;
  Database database_;
  mutable std::map<Filename, std::shared_ptr<loot::Plugin>> plugins_;
};
}

#endif

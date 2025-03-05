use std::{
    fs::{File, copy, create_dir_all},
    path::{Path, PathBuf, absolute},
    time::{Duration, SystemTime},
};

use crate::GameType;
use tempfile::TempDir;

const BLANK_ESM: &str = "Blank.esm";
const BLANK_DIFFERENT_ESM: &str = "Blank - Different.esm";
const BLANK_MASTER_DEPENDENT_ESM: &str = "Blank - Master Dependent.esm";
const BLANK_DIFFERENT_MASTER_DEPENDENT_ESM: &str = "Blank - Different Master Dependent.esm";
const BLANK_ESP: &str = "Blank.esp";
const BLANK_DIFFERENT_ESP: &str = "Blank - Different.esp";
const BLANK_MASTER_DEPENDENT_ESP: &str = "Blank - Master Dependent.esp";
const BLANK_DIFFERENT_MASTER_DEPENDENT_ESP: &str = "Blank - Different Master Dependent.esp";
const BLANK_PLUGIN_DEPENDENT_ESP: &str = "Blank - Plugin Dependent.esp";
const BLANK_DIFFERENT_PLUGIN_DEPENDENT_ESP: &str = "Blank - Different Plugin Dependent.esp";

const BLANK_FULL_ESM: &str = "Blank.full.esm";
const BLANK_MEDIUM_ESM: &str = "Blank.medium.esm";
const BLANK_ESL: &str = "Blank.esl";
const NON_PLUGIN_FILE: &str = "NotAPlugin.esm";

fn source_plugins_path(game_type: GameType) -> PathBuf {
    match game_type {
        GameType::TES3 | GameType::OpenMW => absolute("./testing-plugins/Morrowind/Data Files"),
        GameType::TES4 => absolute("./testing-plugins/Oblivion/Data"),
        GameType::Starfield => absolute("./testing-plugins/Starfield/Data"),
        GameType::FO3 | GameType::FONV | GameType::TES5 => {
            absolute("./testing-plugins/Skyrim/Data")
        }
        _ => absolute("./testing-plugins/SkyrimSE/Data"),
    }
    .unwrap()
}

fn master_file(game_type: GameType) -> &'static str {
    match game_type {
        GameType::TES3 | GameType::OpenMW => "Morrowind.esm",
        GameType::TES4 => "Oblivion.esm",
        GameType::TES5 | GameType::TES5SE | GameType::TES5VR => "Skyrim.esm",
        GameType::FO3 => "Fallout3.esm",
        GameType::FONV => "FalloutNV.esm",
        GameType::FO4 | GameType::FO4VR => "Fallout4.esm",
        GameType::Starfield => "Starfield.esm",
    }
}

fn copy_file(source_dir: &Path, dest_dir: &Path, filename: &str) {
    copy(source_dir.join(filename), dest_dir.join(filename)).unwrap();
}

fn touch(file_path: &Path) {
    std::fs::File::create(file_path).unwrap();
}

fn supports_light_plugins(game_type: GameType) -> bool {
    matches!(
        game_type,
        GameType::TES5SE | GameType::TES5VR | GameType::FO4 | GameType::FO4VR | GameType::Starfield
    )
}

fn is_load_order_timestamp_based(game_type: GameType) -> bool {
    matches!(
        game_type,
        GameType::TES3 | GameType::TES4 | GameType::FO3 | GameType::FONV
    )
}

fn initial_load_order(game_type: GameType) -> Vec<(&'static str, bool)> {
    if game_type == GameType::Starfield {
        vec![
            (master_file(game_type), true),
            (BLANK_ESM, true),
            (BLANK_DIFFERENT_ESM, false),
            (BLANK_FULL_ESM, false),
            (BLANK_MASTER_DEPENDENT_ESM, false),
            (BLANK_MEDIUM_ESM, false),
            (BLANK_ESL, false),
            (BLANK_ESP, false),
            (BLANK_DIFFERENT_ESP, false),
            (BLANK_MASTER_DEPENDENT_ESP, false),
        ]
    } else {
        let mut load_order = vec![
            (master_file(game_type), true),
            (BLANK_ESM, true),
            (BLANK_DIFFERENT_ESM, false),
            (BLANK_MASTER_DEPENDENT_ESM, false),
            (BLANK_DIFFERENT_MASTER_DEPENDENT_ESM, false),
            (BLANK_ESP, false),
            (BLANK_DIFFERENT_ESP, false),
            (BLANK_MASTER_DEPENDENT_ESP, false),
            (BLANK_DIFFERENT_MASTER_DEPENDENT_ESP, true),
            (BLANK_PLUGIN_DEPENDENT_ESP, false),
            (BLANK_DIFFERENT_PLUGIN_DEPENDENT_ESP, false),
        ];

        if supports_light_plugins(game_type) {
            load_order.insert(5, (BLANK_ESL, false));
        }

        load_order
    }
}

fn set_load_order(
    game_type: GameType,
    data_path: &Path,
    local_path: &Path,
    load_order: &[(&'static str, bool)],
) {
    use std::io::Write;

    match game_type {
        GameType::TES3 => {}
        GameType::OpenMW => {}
        _ => {
            let mut file = File::create(local_path.join("Plugins.txt")).unwrap();
            for (plugin, is_active) in load_order {
                if supports_light_plugins(game_type) {
                    if *is_active {
                        write!(file, "*").unwrap();
                    }
                } else if !is_active {
                    continue;
                }

                writeln!(file, "{plugin}").unwrap();
            }
        }
    }

    if is_load_order_timestamp_based(game_type) {
        let mut mod_time = SystemTime::now();
        for (plugin, _) in load_order {
            let ghosted_path = data_path.join(plugin.to_string() + ".ghost");
            let file = if ghosted_path.exists() {
                File::options().write(true).open(ghosted_path)
            } else {
                File::options().write(true).open(data_path.join(plugin))
            };
            file.unwrap().set_modified(mod_time).unwrap();

            mod_time += Duration::from_secs(60);
        }
    } else if game_type == GameType::TES5 {
        let mut file = File::create(local_path.join("loadorder.txt")).unwrap();
        for (plugin, _) in load_order {
            writeln!(file, "{plugin}").unwrap();
        }
    }
}

pub struct Fixture {
    _temp_dir: TempDir,
    pub game_type: GameType,
    pub game_path: PathBuf,
    pub local_path: PathBuf,
}

impl Fixture {
    pub fn new(game_type: GameType) -> Self {
        let temp_dir = tempfile::Builder::new()
            .prefix("libloot-t\u{00E9}st-")
            .tempdir()
            .unwrap();
        let root_path = temp_dir.path();
        let game_path = root_path.join("games/game");
        let local_path = root_path.join("local/game");
        let data_path = match game_type {
            GameType::OpenMW => game_path.join("resources/vfs"),
            GameType::TES3 => game_path.join("Data Files"),
            _ => game_path.join("Data"),
        };

        create_dir_all(&data_path).unwrap();
        create_dir_all(&local_path).unwrap();

        let source_plugins_path = source_plugins_path(game_type);

        if game_type == GameType::Starfield {
            copy_file(&source_plugins_path, &data_path, BLANK_FULL_ESM);
            copy_file(&source_plugins_path, &data_path, BLANK_MEDIUM_ESM);

            copy(
                source_plugins_path.join(BLANK_FULL_ESM),
                data_path.join(BLANK_ESM),
            )
            .unwrap();
            copy(
                source_plugins_path.join(BLANK_FULL_ESM),
                data_path.join(BLANK_DIFFERENT_ESM),
            )
            .unwrap();
            copy(
                source_plugins_path.join("Blank - Override.full.esm"),
                data_path.join(BLANK_MASTER_DEPENDENT_ESM),
            )
            .unwrap();
            copy_file(&source_plugins_path, &data_path, BLANK_ESP);
            copy(
                source_plugins_path.join(BLANK_ESP),
                data_path.join(BLANK_DIFFERENT_ESP),
            )
            .unwrap();
            copy(
                source_plugins_path.join("Blank - Override.esp"),
                data_path.join(BLANK_MASTER_DEPENDENT_ESP),
            )
            .unwrap();
        } else {
            copy_file(&source_plugins_path, &data_path, BLANK_ESM);
            copy_file(&source_plugins_path, &data_path, BLANK_DIFFERENT_ESM);
            copy_file(&source_plugins_path, &data_path, BLANK_MASTER_DEPENDENT_ESM);
            copy_file(
                &source_plugins_path,
                &data_path,
                BLANK_DIFFERENT_MASTER_DEPENDENT_ESM,
            );
            copy_file(&source_plugins_path, &data_path, BLANK_ESP);
            copy_file(&source_plugins_path, &data_path, BLANK_DIFFERENT_ESP);
            copy_file(&source_plugins_path, &data_path, BLANK_MASTER_DEPENDENT_ESP);
            copy_file(
                &source_plugins_path,
                &data_path,
                BLANK_DIFFERENT_MASTER_DEPENDENT_ESP,
            );
            copy_file(&source_plugins_path, &data_path, BLANK_PLUGIN_DEPENDENT_ESP);
            copy_file(
                &source_plugins_path,
                &data_path,
                BLANK_DIFFERENT_PLUGIN_DEPENDENT_ESP,
            );
        }

        if supports_light_plugins(game_type) {
            if game_type == GameType::Starfield {
                copy(
                    source_plugins_path.join("Blank.small.esm"),
                    data_path.join(BLANK_ESL),
                )
                .unwrap();
            } else {
                copy_file(&source_plugins_path, &data_path, BLANK_ESL);
            }
        }

        let master_file = master_file(game_type);
        copy(data_path.join(BLANK_ESM), data_path.join(master_file)).unwrap();

        set_load_order(
            game_type,
            &data_path,
            &local_path,
            &initial_load_order(game_type),
        );

        if game_type == GameType::OpenMW {
            touch(&game_path.join("openmw.cfg"));
        } else {
            std::fs::rename(
                data_path.join(BLANK_MASTER_DEPENDENT_ESM),
                data_path.join(BLANK_MASTER_DEPENDENT_ESM.to_string() + ".ghost"),
            )
            .unwrap();
        }

        std::fs::write(
            data_path.join(NON_PLUGIN_FILE),
            "This isn't a valid plugin file.",
        )
        .unwrap();

        Self {
            _temp_dir: temp_dir,
            game_type,
            game_path,
            local_path,
        }
    }
}

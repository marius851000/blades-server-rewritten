from pathlib import Path
from typing import Dict, List
import os
import json
import re
import yaml
from resolve_refs import resolve_refs
import parse_util
import argparse

QUESTDATA_GUID = "5cc072dcf7a2ad3459c439adc756d484"
LEVELDATA_PATH = "Assets/export/gameplaymetadata/LevelData.asset"
INTERACTABLE_ITEM_PATH = "Assets/MonoBehaviour/InteractableItemData.asset"
COMMON_PATH = "Assets/BGS/Scenes/common.unity"


# from gpt-oss:20b
def find_all_case_insensitive(
    root: Path | str,
    rel_path: Path | str,
) -> Path:
    """
    Recursively search *root* for *rel_path* treating each component
    case‑insensitively.  Returns the first matching full path.
    Raises FileNotFoundError if no match is found.

    Parameters
    ----------
    root : Path or str
        Absolute path to the directory that will be searched.
    rel_path : Path or str
        Relative path (e.g. "Assets/BGS/Levels/Arena.asset").

    Returns
    -------
    Path
        The first full path that matches the case‑insensitive search.
    """
    root = Path(root).resolve()
    parts = Path(rel_path).parts

    def _recurse(current: Path, remaining: List[str]) -> List[Path]:
        if not remaining:
            return [current] if current.exists() else []

        part = remaining[0]
        # Find *all* children that match the current component ignoring case.
        candidates = [
            p for p in current.iterdir()
            if p.name.lower() == part.lower()
        ]

        results: List[Path] = []
        for cand in candidates:
            results.extend(_recurse(cand, remaining[1:]))
        return results

    matches = _recurse(root, list(parts))
    if not matches:
        raise FileNotFoundError(f"No file matching {rel_path!r} found under {root!r}")
    return matches[0]


class UnityAssetMeta:
    def __init__(self, meta, asset_path):
        self.guid = meta["guid"]
        self.path = asset_path

class UnityAssets:
    def __init__(self, folder):
        self.base_folder = folder
        self.guid_to_info: Dict[str, UnityAssetMeta] = {}
        for path, dirs, files in os.walk(folder):
            for file in files:
                if file.endswith(".asset"):
                    file_path = os.path.join(path, file)
                    meta_path = file_path + ".meta"
                    try:
                        with open(meta_path, "r") as f:
                            meta_data = yaml.safe_load(f.read())
                    except FileNotFoundError:
                        continue
                    self.guid_to_info[meta_data["guid"]] = UnityAssetMeta(meta_data, file_path)



class Analyzer:
    def __init__(self, assets: UnityAssets):
        pass

def unity_tag_constructor(loader, node):
    return loader.construct_mapping(node)

yaml.SafeLoader.add_constructor(None, unity_tag_constructor)

if __name__ == "__main__":
    parser = argparse.ArgumentParser(prog='BladeDataExporter')
    parser.add_argument('decompiled_apk_unity_data_root')
    parser.add_argument('output_json')
    # valid option are "item", "interactable", "quest" and "dungeon"
    # This is mostly meant for faster iteration in development. You should typically do the whole process in CI or the like
    # (which is done if no arg is provided)
    parser.add_argument('-p','--parse', action='append')
    args = parser.parse_args()


    source_path = args.decompiled_apk_unity_data_root
    out_path = args.output_json
    to_parse = args.parse
    parse_all = to_parse is None
    if to_parse is None:
        to_parse = []
    else:
        parse_all = len(to_parse) == 0

    print("loading assets")
    assets = UnityAssets(source_path)
    to_export = {}
    if not parse_all:
        with open(out_path, "r") as f:
            to_export = json.loads(f.read())

    if ("item" in to_parse) or parse_all:
        print("loading item template")
        items_template = {}
        with open(os.path.join(source_path, COMMON_PATH), "r") as f_common:
            content = f_common.read()
            # Remove Unity specific tags (!u! followed by numbers) before parsing. Needed for some reason.
            cleaned_content = re.sub(r'!u!\d+', '', content)
            for yaml_sub_file in yaml.safe_load_all(cleaned_content):
                if "MonoBehaviour" in yaml_sub_file and "_itemTemplateLists" in yaml_sub_file["MonoBehaviour"]:
                    print("loading inventory items")
                    for item_template_list in yaml_sub_file["MonoBehaviour"]["_itemTemplateLists"]:
                        item_template_list_guid = item_template_list["guid"]
                        with open(assets.guid_to_info[item_template_list_guid].path, "r") as f_item_template_list:
                            item_template_list_data = yaml.safe_load(f_item_template_list.read())
                            for item_template_source in item_template_list_data["MonoBehaviour"]["_templateList"]:
                                items_template[item_template_source["_uid"]["_id"]] = {
                                    "name": item_template_source["_name"]["_key"],
                                    "type": item_template_source["_type"]
                                }

        assert items_template != {}
        to_export["items_template"] = items_template
    else:
        print("not parsing 'item' template")

    if ("interactable" in to_parse) or parse_all:
        print("loading interactable items")
        interactable_definition_by_uuid = {}
        with open(os.path.join(source_path, INTERACTABLE_ITEM_PATH), "r") as f_interactables:
            interactable_data = yaml.safe_load(f_interactables.read())
            for interactable_list_entry in interactable_data["MonoBehaviour"]["_keyItemDataList"]:
                interactable_uuid = interactable_list_entry["Key"]
                interactable_guid = interactable_list_entry["ItemData"]["guid"]
                with open(assets.guid_to_info[interactable_guid].path, "r") as f_this_interactable:
                    this_interactable_data = yaml.safe_load(f_this_interactable.read())
                    loot_table = {}
                    for loot_entry in this_interactable_data["MonoBehaviour"]["_lootTableList"]:
                        loot_table[loot_entry["_lootTableId"]["_uid"]["_id"]] = {}
                    interactable_definition_by_uuid[interactable_uuid] = {
                        "loot_table": loot_table
                    }
        to_export["interactables"] = interactable_definition_by_uuid
    else:
        print("not parsing 'interactable' items")

    if "quest" in to_parse or parse_all:
        print("loading quest data id mapping")
        quest_definition_uuid_to_guid = {}
        with open(assets.guid_to_info[QUESTDATA_GUID].path, "r") as f_leveldata:
            quest_data = yaml.safe_load(f_leveldata.read())
            for yaml_sub_file in quest_data["MonoBehaviour"]["_keyQuestList"]:
                quest_definition_uuid_to_guid[yaml_sub_file["Key"]] = yaml_sub_file["QuestValue"]["guid"]

        print("loading quest definition data themself")
        quest_definition_by_uuid = {}
        for quest_uuid, quest_guid in quest_definition_uuid_to_guid.items():
            with open(assets.guid_to_info[quest_guid].path, "r") as f_leveldata:
                quest_definition_data_container = yaml.safe_load(f_leveldata.read())
            quest_definition_data = resolve_refs(json.loads(quest_definition_data_container["MonoBehaviour"]["_serializedJsonString"]))
            quest_dungeon_info = None
            if "_dungeonQuest" in quest_definition_data:
                quest_objectives = {}
                for objective in quest_definition_data["_dungeonQuest"]["_objectives"]:
                    quest_objectives[objective["_uid"]["_id"]] = {
                        "description": objective["Description"]["_key"],
                        "quota": objective["_quota"],
                        "rewards": parse_util.parse_reward_givers(objective["RewardGivers"])
                    }
                quest_dungeon_info = {
                    "objectives" : quest_objectives,
                    "version": quest_definition_data["_dungeonQuest"]["_questVersion"],
                    "dungeon_uuid": quest_definition_data["_dungeonQuest"]["DungeonSettingsPointer"]["_uid"]["_id"]
                }
            quest_definition_by_uuid[quest_uuid] = {
                "dungeon_info": quest_dungeon_info,
                #"raw_data": quest_definition_data
            }
        to_export["quests"] = quest_definition_by_uuid
    else:
        print("not parsing 'quest'")

    if "dungeon" in to_parse or parse_all:
        print("loading dungeon settings")
        dungeon_settings_by_uuid = {}
        with open(os.path.join(source_path, LEVELDATA_PATH), "r") as f_leveldata:
            level_data_list_file = yaml.safe_load(f_leveldata.read())
            for level_meta in level_data_list_file["MonoBehaviour"]["_levelList"]:
                level_uuid = level_meta["Id"]
                level_handle = level_meta["ResourceHandle"]
                level_path = level_meta["Path"]
                with open(find_all_case_insensitive(source_path, level_path), "r") as f_thislevel:
                    raw_level_data = yaml.safe_load(f_thislevel.read())
                    chest_spawn_info = {}
                    for chest in raw_level_data["MonoBehaviour"]["_settings"]["_spawnSettings"]["_spawnGroupsChest"]:
                        chest_spawn_info[chest["_uid"]["_id"]] = {}
                    item_spawn_info = {}
                    for item in raw_level_data["MonoBehaviour"]["_settings"]["_spawnSettings"]["_spawnGroupsItem"]:
                        item_spawn_info[item["_uid"]["_id"]] = {
                            "name": item["_name"],
                            "apparition_settings": parse_util.parse_apparition_settings(item["_apparitionSettings"])
                        }
                    enemy_spawn_info = {}
                    for enemy in raw_level_data["MonoBehaviour"]["_settings"]["_spawnSettings"]["_spawnGroupsEnemy"]:
                        enemy_spawn_info[enemy["_uid"]["_id"]] = {
                            "quantity": enemy["_quantity"]
                        }

                    dungeon_settings_by_uuid[level_uuid] = {
                        "handle": level_handle,
                        "spawn_info": {
                            "chest": chest_spawn_info,
                            "item": item_spawn_info,
                            "enemy_spawn_groups": enemy_spawn_info
                        },
                        #"raw_data": raw_level_data
                    }
        to_export["dungeons"] = dungeon_settings_by_uuid
    else:
        print("not parsing 'dungeon'")

    print("exporting...")
    with open(out_path, "w") as f:
        f.write(json.dumps(to_export, indent="\t"))

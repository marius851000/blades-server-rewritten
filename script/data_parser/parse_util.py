from typing import List

def parse_apparition_settings(settings: List[dict]) -> List[dict]:
    result = []
    for entry in settings:
        result.append({
            "interactable_uuid": entry["_clutterType"]["_uid"]["_id"],
            "weight": entry["_weightMultiplier"],
            "mandatory": entry["_mandatory"], # not sure what that represent, but it seems important
        })
    return result

def parse_reward_givers(reward_givers: List[dict]) -> List[dict]:
    result = []
    for entry in reward_givers:
        result.append(parse_reward(entry["_reward"]))
    return result

def parse_reward(reward: dict) -> dict:
    items_to_reward = []
    for item_reward_entry in reward["_inventory"]["_itemsList"]:
        items_to_reward.append({
            "count": item_reward_entry["_count"],
            "template_uuid": item_reward_entry["Item"]["_uid"]["_id"]
        })
    return {
        "experience": reward["_experience"],
        "town_points": reward["_townPoints"],
        "chest_is_none": reward["_chest"]["_chestCycle"]["_uid"]["_id"] == 0, # just so the server can panic if the data is invalid
        "items_to_reward": items_to_reward
    }

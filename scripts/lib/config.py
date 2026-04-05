import os
from scripts.lib import yaml

def get_config(repo_root):
    """
    Returns the loaded configuration object from .claude/settings.json or yaml.
    Fallback default is provided.
    """
    config_path = os.path.join(repo_root, ".claude", "settings.json")
    if os.path.exists(config_path):
        import json
        with open(config_path, "r") as f:
            conf = json.load(f)
            if "agents" not in conf:
                conf["agents"] = {"audit_log": "logs/audit.jsonl"}
            elif "audit_log" not in conf["agents"]:
                conf["agents"]["audit_log"] = "logs/audit.jsonl"
            return conf

    return {
        "agents": {
            "audit_log": "logs/audit.jsonl"
        }
    }

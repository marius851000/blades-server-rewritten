"""
Utility to resolve Unity‑style reference dictionaries.

In Unity exported assets the *source* reference is stored under a ``$ref`` key.
The *target* asset that defines the reference contains a ``$id`` key and an
``_id`` field that uniquely identifies the asset.  This helper scans the
entire data structure for all such target objects and replaces any
reference that contains a ``$ref`` key but no ``_id`` with the fully
defined target dictionary.

The function operates on a single parsed asset (a mapping) and returns
a new mapping with all references resolved.  It does not modify the
original input.
"""

# made by gpt-oss:20b

from __future__ import annotations

from typing import Any, Dict, Mapping, cast

__all__ = ["resolve_refs"]


def resolve_refs(data: Mapping[str, Any]) -> Dict[str, Any]:
    """
    Resolve Unity reference dictionaries in a single object.

    Parameters
    ----------
    data : Mapping[str, Any]
        The Unity object to process.  The function does not modify the
        input; a new mapping is returned.

    Returns
    -------
    dict
        A new mapping with all ``$ref`` placeholders replaced by the
        fully defined dictionary that contains the matching ``$id``.
    """
    if not isinstance(data, Mapping):
        raise TypeError("resolve_refs expects a mapping (dict)")

    # Build a lookup for all fully defined references in the entire data
    # structure.  Any dictionary that contains both a "$id" key and an
    # "_id" key is considered a target.  The key in the map is the "$id"
    # value.
    ref_map: Dict[str, Dict[str, Any]] = {}

    def _collect_targets(obj: Any) -> None:
        if isinstance(obj, Mapping):
            if "$id" in obj and "_id" in obj:
                ref_map[obj["$id"]] = obj
            for v in obj.values():
                _collect_targets(v)
        elif isinstance(obj, list):
            for v in obj:
                _collect_targets(v)

    _collect_targets(data)

    # Recursive resolver that walks the data structure.
    def _resolve(obj: Any) -> Any:
        if isinstance(obj, Mapping):
            # Replace a reference that uses "$ref" and lacks an "_id"
            if "$ref" in obj and "_id" not in obj:
                ref = obj["$ref"]
                if ref in ref_map:
                    return _resolve(ref_map[ref])
            # Walk into mapping
            return {k: _resolve(v) for k, v in obj.items()}

        if isinstance(obj, list):
            return [_resolve(v) for v in obj]

        return obj

    # Return a new dict; we don't want to mutate the original
    return cast(Dict[str, Any], _resolve(data))

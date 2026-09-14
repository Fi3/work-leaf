"""Qualify selected outer-turn catalog boundaries, without usage accounting.

The catalog is a replaceable developer section. Its last observed value carries
across turns without a replacement. Comparison requires explicit selected IDs;
catalogs from later review/repair turns cannot qualify an earlier author turn.
The caller owns semantic boundary eligibility; ordinal equality alone cannot
establish a causal comparison. Other developer sections require separate checks.
"""

import hashlib
import re


CATALOG = re.compile(r"<skills_instructions>.*?</skills_instructions>", re.S)


def _boundaries(rows, selected_ids):
    errors = []
    if not selected_ids or len(set(selected_ids)) != len(selected_ids):
        errors.append("selected turn IDs must be nonempty and unique")
    selected = set(selected_ids)
    found = {}
    order = []
    current = None
    profile = None
    model_started = False
    for lineno, row in enumerate(rows, 1):
        payload = row.get("payload", {})
        if row.get("type") == "event_msg" and payload.get("type") == "task_started":
            current = payload.get("turn_id")
            model_started = False
            if current in selected:
                if current in found:
                    errors.append(f"duplicate native task_started: {current}")
                order.append(current)
                found[current] = {"turn_id": current, "start_line": lineno,
                                  "catalog": profile}
        if row.get("type") != "response_item":
            continue
        if payload.get("role") == "developer":
            text = "\n".join(item.get("text", "") for item in payload.get("content", []))
            sections = CATALOG.findall(text)
            if len(sections) > 1:
                errors.append(f"ambiguous catalog sections at line {lineno}")
            if sections:
                if model_started and current in selected:
                    errors.append(f"catalog changed after model output at line {lineno}")
                profile = sections[-1]
                if current in selected:
                    found[current]["catalog"] = profile
                    found[current]["catalog_line"] = lineno
        elif payload.get("role") == "assistant" or payload.get("type") in (
            "function_call", "reasoning", "custom_tool_call"
        ):
            model_started = True
    if order != list(selected_ids):
        errors.append("requested turns are missing or not in native source order")
    result = []
    for turn_id in selected_ids:
        item = found.get(turn_id)
        if item is None:
            continue
        if item["catalog"] is None:
            errors.append(f"no inherited or explicit catalog for {turn_id}")
        result.append(item)
    return result, errors


def compare_catalog_boundaries(reference_rows, actual_rows, reference_ids, actual_ids):
    """Compare exact effective sections on explicit ordered outer-turn mappings."""
    expected, reference_errors = _boundaries(reference_rows, reference_ids)
    actual, actual_errors = _boundaries(actual_rows, actual_ids)
    errors = (["reference: " + error for error in reference_errors]
              + ["actual: " + error for error in actual_errors])
    if len(reference_ids) != len(actual_ids):
        errors.append("every actual turn requires an explicit eligible reference boundary")
    pairs = []
    for left, right in zip(expected, actual):
        def public(item):
            catalog = item["catalog"]
            return {**{k: v for k, v in item.items() if k != "catalog"},
                    "catalog_bytes": len(catalog.encode()) if catalog is not None else None,
                    "catalog_sha256": hashlib.sha256(catalog.encode()).hexdigest()
                    if catalog is not None else None}
        pairs.append({"reference": public(left), "actual": public(right),
                      "equal": left["catalog"] is not None
                      and left["catalog"] == right["catalog"]})
    return {"matches": not errors and bool(pairs) and all(pair["equal"] for pair in pairs),
            "pairs": pairs, "errors": errors}

"""Disjoint stage accounting; recorded partial attempts are not complete effects."""
from fractions import Fraction

CATEGORIES = ("initial_author", "author_fix", "review", "integration")


def fraction(value):
    value = Fraction(value)
    return {"numerator": value.numerator, "denominator": value.denominator}


def partition(ledger):
    records = ledger["records"]
    assigned, values = set(), dict.fromkeys(CATEGORIES, 0)
    for turn in ledger["turns"]:
        category = turn["category"]
        if category not in values:
            raise ValueError("unknown stage category")
        for identity in turn["native_response_ids"]:
            if identity not in records or identity in assigned:
                raise ValueError("missing or duplicate stage response ownership")
            assigned.add(identity)
            usage = records[identity]["usage"]
            for key in ("input_tokens", "output_tokens"):
                if type(usage.get(key)) is not int or usage[key] < 0:
                    raise ValueError("invalid response usage")
            values[category] += usage["input_tokens"] + usage["output_tokens"]
    if assigned != set(records):
        raise ValueError("observed response has no stage owner")
    if sum(values.values()) != ledger["native_usage"]["raw_input_plus_output"]:
        raise ValueError("whole response total differs from partition")
    for category, value in values.items():
        declared = ledger["category_totals"].get(category, {}).get("native_usage", {}).get("raw_input_plus_output", 0)
        if value != declared:
            raise ValueError("stage declaration differs from response partition")
    return values


def summarize(reference, runs):
    if len(runs) != 3 or len({r["run_id"] for r in runs}) != 3:
        raise ValueError("exactly three distinct admitted outcomes are required")
    reference_values = partition(reference)
    seen, partitions = set(reference["records"]), []
    for run in runs:
        if seen & set(run["records"]):
            raise ValueError("response belongs to more than one comparison population")
        seen.update(run["records"])
        partitions.append(partition(run))
    complete = all(
        x["status"] == "qualified_observed_ledger" and not x["errors"]
        and x["workflow_result"] == "pass"
        and not x["capture_qualification"]["unknown_unfinished_tail_observed"]
        for x in [reference, *runs])
    mean = Fraction(sum(r["native_usage"]["raw_input_plus_output"] for r in runs), 3)
    net = mean - reference["native_usage"]["raw_input_plus_output"]
    stage = {c:Fraction(sum(v[c] for v in partitions),3)-reference_values[c] for c in CATEGORIES}
    if sum(stage.values()) != net:
        raise ValueError("stage offsets do not reconcile to whole net")
    return {
        "scope":"saved-reference joint package; not historical causal attribution",
        "outcomes":[{"run_id":r["run_id"],"workflow_result":r["workflow_result"],
            "accounting_status":r["status"],"recorded_raw":r["native_usage"]["raw_input_plus_output"],
            "unknown_unfinished_tail":r["capture_qualification"]["unknown_unfinished_tail_observed"]}
            for r in runs],
        "complete_workflow_contrast":complete,
        "recorded_attempt_mean_raw":fraction(mean),
        "recorded_reference_raw":reference["native_usage"]["raw_input_plus_output"],
        "net_delta_raw":fraction(net) if complete else None,
        "stage_delta_raw":{c:fraction(v) for c,v in stage.items()} if complete else None,
        "recorded_stage_differences_not_causal":{c:fraction(v) for c,v in stage.items()},
        "historical_explained_share":None,
        "limit":"Complete accounting and stage offsets do not establish mechanism activation, equivalent outputs, statistical certainty or historical transfer. Partial recorded means are not full-workflow treatment effects."}

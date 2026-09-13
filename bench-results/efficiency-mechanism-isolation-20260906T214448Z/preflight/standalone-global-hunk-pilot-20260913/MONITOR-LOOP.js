const phase = load("global_P");
const sampler = load("global_resource_cmd");
const launchCommand = load("global_launch_command");
if (!phase || !sampler || !launchCommand) throw new Error("Missing frozen launch inputs");
function add(path, value) {
  const body = typeof value === "string" ? value : JSON.stringify(value, null, 2) + "\n";
  return "*** Begin Patch\n*** Add File: " + path + "\n" +
    body.replace(/\n$/, "").split("\n").map(x => "+" + x).join("\n") + "\n*** End Patch";
}
const launch = await tools.exec_command({cmd: launchCommand, yield_time_ms: 1000, max_output_tokens: 1500});
store("global_launch_result", launch);
store("global_supervisor", launch.session_id ?? null);
let cancelled = false;
let priorTime = null;
let sequence = 0;
let launchReceiptSaved = false;
async function cancel(reason) {
  if (cancelled || !launch.session_id) return;
  const result = await tools.write_stdin({session_id: launch.session_id, chars: "\u0003",
    yield_time_ms: 1000, max_output_tokens: 1500});
  cancelled = true;
  store("global_cancel_result", result);
  await tools.apply_patch(add(phase + "/OPERATOR-RESOURCE-STOP.json",
    {reason, at_utc: new Date().toISOString(), result}));
}
while (true) {
  try {
    if (!launchReceiptSaved) {
      await tools.apply_patch(add(phase + "/SUPERVISOR-TOOL-LAUNCH.json", launch));
      launchReceiptSaved = true;
    }
    const result = await tools.exec_command({cmd: sampler, yield_time_ms: 1000, max_output_tokens: 12000});
    if (result.exit_code !== 0) throw new Error("Sampler failed: " + JSON.stringify(result));
    const sample = JSON.parse(result.output);
    sequence++;
    const instant = Date.parse(sample.at_utc);
    const gap = priorTime === null ? null : (instant - priorTime) / 1000;
    priorTime = instant;
    sample.monitor_sequence = sequence;
    sample.gap_seconds = gap;
    store("global_last_resource", sample);
    await tools.apply_patch(add(phase + "/resource-samples/sample-" + String(sequence).padStart(4,"0") + ".json", sample));
    const lineRead = await tools.exec_command({cmd: "rg '^\\*\\*Live resource monitor:' /home/user/src/work-leaf/ephemeral-note.md",
      yield_time_ms: 1000, max_output_tokens: 1000});
    if (lineRead.exit_code !== 0) throw new Error("Live-note anchor missing");
    const old = lineRead.output.trimEnd();
    if (old.split("\n").length !== 1) throw new Error("Ambiguous live-note anchor");
    const row = sample.runs[0];
    const status = sample.phase_terminal ? "terminal" : "active";
    const live = "**Live resource monitor:** " + sample.at_utc + "; global-hunk supervisor " +
      (launch.session_id ?? "already terminal") + " " + status + "; " + row.completed_public_turns +
      " completed public turns / " + row.reported_raw + " observed raw; unreconciled, not a saving.";
    await tools.apply_patch("*** Begin Patch\n*** Update File: /home/user/src/work-leaf/ephemeral-note.md\n@@\n-" +
      old + "\n+" + live + "\n*** End Patch");
    notify({sequence, at_utc: sample.at_utc, raw: row.reported_raw,
      completed_public_turns: row.completed_public_turns, phase_terminal: sample.phase_terminal, gap_seconds: gap});
    if (sample.stop_wave && !sample.phase_terminal) await cancel("Observed completed-public raw >= 25000000");
    if (gap !== null && gap > 60 && !sample.phase_terminal) await cancel("Resource sampling gap exceeded 60 seconds");
    if (sample.phase_terminal || !launch.session_id) break;
    await new Promise(resolve => setTimeout(resolve, 30000));
  } catch (error) {
    let controlError = null;
    try { await cancel("Monitoring failure: " + String(error)); }
    catch (stopError) { controlError = String(stopError); }
    const failure = {at_utc: new Date().toISOString(), error: String(error), sequence,
      control_error: controlError, supervisor_session: launch.session_id ?? null};
    store("global_monitor_state", {active:false, failed:true, ...failure});
    try { await tools.apply_patch(add(phase + "/MONITOR-ERROR.json", failure)); }
    catch (publicationError) { failure.publication_error = String(publicationError); }
    text({monitor_failed: true, ...failure});
    exit();
  }
}
let terminal = launch;
if (launch.session_id) {
  terminal = await tools.write_stdin({session_id: launch.session_id, chars:"", yield_time_ms:1000, max_output_tokens:4000});
}
store("global_terminal_tool_result", terminal);
store("global_monitor_state", {active:false, sequence, terminal});
await tools.apply_patch(add(phase + "/SUPERVISOR-TOOL-TERMINAL.json", terminal));
text({monitor_complete:true, sequence, terminal});

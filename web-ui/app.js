const ENDPOINTS = {
  state: "/state",
  events: "/events/drain",
  command: "/command",
  commandAgent: "/command-agent",
  agentMessage: "/agent/message",
  agentInterrupt: "/agent/interrupt",
  transcript: "/transcript",
  loadingText: "/loading-text",
  shutdown: "/shutdown",
};

const POLL_MS = 900;
const STATE_REFRESH_MS = 5000;

const state = {
  apiBase: "",
  connected: false,
  busy: false,
  selectedAgentId: null,
  snapshot: {
    command_transcript: [],
    sessions: [],
  },
  lastStateRefresh: 0,
  pollTimer: 0,
  requestInFlight: false,
};

const dom = {};

document.addEventListener("DOMContentLoaded", () => {
  bindDom();
  bindEvents();
  state.apiBase = defaultApiBase();
  dom.apiBase.value = state.apiBase;
  render();

  if (state.apiBase) {
    runAction(connect);
  }
});

function bindDom() {
  dom.apiBase = document.querySelector("#apiBase");
  dom.connectButton = document.querySelector("#connectButton");
  dom.refreshButton = document.querySelector("#refreshButton");
  dom.shutdownButton = document.querySelector("#shutdownButton");
  dom.connectionStatus = document.querySelector("#connectionStatus");
  dom.busyStatus = document.querySelector("#busyStatus");
  dom.notice = document.querySelector("#notice");
  dom.commandSurface = document.querySelector("#commandSurface");
  dom.sessionList = document.querySelector("#sessionList");
  dom.launchPrompt = document.querySelector("#launchPrompt");
  dom.launchButton = document.querySelector("#launchButton");
  dom.commandLine = document.querySelector("#commandLine");
  dom.commandButton = document.querySelector("#commandButton");
  dom.reviewButton = document.querySelector("#reviewButton");
  dom.linearizeButton = document.querySelector("#linearizeButton");
  dom.threadTitle = document.querySelector("#threadTitle");
  dom.threadMeta = document.querySelector("#threadMeta");
  dom.threadStatus = document.querySelector("#threadStatus");
  dom.interruptButton = document.querySelector("#interruptButton");
  dom.decisionBar = document.querySelector("#decisionBar");
  dom.acceptDoneButton = document.querySelector("#acceptDoneButton");
  dom.keepOpenButton = document.querySelector("#keepOpenButton");
  dom.transcript = document.querySelector("#transcript");
  dom.messageInput = document.querySelector("#messageInput");
  dom.sendButton = document.querySelector("#sendButton");
}

function bindEvents() {
  dom.connectButton.addEventListener("click", () => runAction(connect));
  dom.refreshButton.addEventListener("click", () => runAction(() => refreshState(true)));
  dom.shutdownButton.addEventListener("click", () => runAction(shutdownController));
  dom.commandSurface.addEventListener("click", selectCommandSurface);
  dom.launchButton.addEventListener("click", () => runAction(launchAgent));
  dom.launchPrompt.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      runAction(launchAgent);
    }
  });
  dom.commandButton.addEventListener("click", () => runAction(runCommandLine));
  dom.commandLine.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      runAction(runCommandLine);
    }
  });
  dom.reviewButton.addEventListener("click", () => runAction(() => executeCommand("review")));
  dom.linearizeButton.addEventListener("click", () => runAction(() => executeCommand("linearize")));
  dom.interruptButton.addEventListener("click", () => runAction(interruptSelectedAgent));
  dom.acceptDoneButton.addEventListener("click", () => runAction(() => sendDecision("yes")));
  dom.keepOpenButton.addEventListener("click", () => runAction(() => sendDecision("no")));
  dom.sendButton.addEventListener("click", () => runAction(sendComposerMessage));
  dom.messageInput.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      runAction(sendComposerMessage);
    }
  });
}

function runAction(action) {
  Promise.resolve(action()).catch((error) => {
    state.connected = false;
    setNotice(error.message, "error");
    render();
  });
}

function defaultApiBase() {
  const params = new URLSearchParams(window.location.search);
  const fromQuery = params.get("api");
  if (fromQuery) {
    return trimTrailingSlash(fromQuery);
  }

  const saved = window.localStorage.getItem("workLeafApiBase");
  if (saved) {
    return trimTrailingSlash(saved);
  }

  if (window.location.protocol === "http:" || window.location.protocol === "https:") {
    return window.location.origin;
  }

  return "";
}

async function connect() {
  const apiBase = trimTrailingSlash(dom.apiBase.value);
  if (!apiBase) {
    setNotice("Enter the orchestrator URL.", "error");
    return;
  }

  state.apiBase = apiBase;
  window.localStorage.setItem("workLeafApiBase", apiBase);
  await refreshState(true);
  startPolling();
}

function startPolling() {
  window.clearInterval(state.pollTimer);
  state.pollTimer = window.setInterval(() => {
    poll().catch((error) => {
      state.connected = false;
      setNotice(error.message, "error");
      render();
    });
  }, POLL_MS);
}

async function poll() {
  if (!state.connected || state.requestInFlight) {
    return;
  }

  state.requestInFlight = true;
  try {
    const events = await postJson(ENDPOINTS.events, {});
    applyEvents(events);
    const shouldRefreshState =
      state.busy || Date.now() - state.lastStateRefresh > STATE_REFRESH_MS;
    if (shouldRefreshState) {
      await refreshState(false);
    } else {
      render();
    }
  } finally {
    state.requestInFlight = false;
  }
}

async function refreshState(showSuccess) {
  try {
    const controllerState = await getJson(ENDPOINTS.state);
    state.connected = true;
    state.busy = Boolean(controllerState.busy);
    state.snapshot = normalizeSnapshot(controllerState.snapshot);
    state.lastStateRefresh = Date.now();
    ensureSelection();
    if (showSuccess) {
      setNotice("Connected.", "ok");
    }
    render();
  } catch (error) {
    state.connected = false;
    render();
    throw error;
  }
}

async function launchAgent() {
  const prompt = dom.launchPrompt.value.trim();
  if (!prompt) {
    dom.launchPrompt.focus();
    return;
  }

  await executeCommand(`new ${prompt}`);
  dom.launchPrompt.value = "";
}

async function runCommandLine() {
  const line = dom.commandLine.value.trim();
  if (!line) {
    dom.commandLine.focus();
    return;
  }

  await executeCommand(line);
  dom.commandLine.value = "";
}

async function executeCommand(line) {
  await postJson(ENDPOINTS.command, { line });
  state.busy = true;
  await drainAfterAction();
}

async function sendComposerMessage() {
  const message = dom.messageInput.value.trim();
  if (!message) {
    dom.messageInput.focus();
    return;
  }

  if (state.selectedAgentId) {
    await postJson(ENDPOINTS.agentMessage, {
      agent_id: state.selectedAgentId,
      message,
    });
  } else {
    await postJson(ENDPOINTS.commandAgent, { message });
  }

  state.busy = true;
  dom.messageInput.value = "";
  await drainAfterAction();
}

async function sendDecision(message) {
  if (!state.selectedAgentId) {
    return;
  }

  await postJson(ENDPOINTS.agentMessage, {
    agent_id: state.selectedAgentId,
    message,
  });
  state.busy = true;
  await drainAfterAction();
}

async function interruptSelectedAgent() {
  if (!state.selectedAgentId) {
    return;
  }

  await postJson(ENDPOINTS.agentInterrupt, {
    agent_id: state.selectedAgentId,
  });
  await drainAfterAction();
}

async function shutdownController() {
  await postJson(ENDPOINTS.shutdown, {});
  window.clearInterval(state.pollTimer);
  state.connected = false;
  state.busy = false;
  setNotice("Shutdown requested.", "ok");
  render();
}

async function drainAfterAction() {
  const events = await postJson(ENDPOINTS.events, {});
  applyEvents(events);
  await refreshState(false);
}

async function getJson(path) {
  return requestJson("GET", path);
}

async function postJson(path, body) {
  return requestJson("POST", path, body);
}

async function requestJson(method, path, body) {
  const options = {
    method,
    headers: {
      Accept: "application/json",
    },
  };
  if (body !== undefined) {
    options.headers["Content-Type"] = "application/json";
    options.body = JSON.stringify(body);
  }

  const response = await window.fetch(`${state.apiBase}${path}`, options);
  const text = await response.text();
  let payload = null;
  if (text) {
    try {
      payload = JSON.parse(text);
    } catch (error) {
      throw new Error(`Invalid JSON from ${path}: ${error.message}`);
    }
  }

  if (!response.ok) {
    throw new Error(payload?.error || `${method} ${path} failed`);
  }

  return payload;
}

function applyEvents(events) {
  if (!Array.isArray(events) || events.length === 0) {
    return;
  }

  for (const event of events) {
    const { variant, payload } = eventVariant(event);
    switch (variant) {
      case "AgentAdded":
      case "AgentUpdated":
        upsertSession(payload.session);
        if (!state.selectedAgentId) {
          state.selectedAgentId = idKey(payload.session.id);
        }
        break;
      case "AgentStatusUpdated":
        upsertSessionStatus(payload);
        break;
      case "AgentUsageUpdated":
        updateSessionUsage(payload.agent_id, payload.token_usage);
        break;
      case "AgentLineAppended":
        appendSessionLine(payload.agent_id, payload.line);
        break;
      case "AgentSelected":
        state.selectedAgentId = idKey(payload.agent_id);
        break;
      case "CommandTranscriptLine":
        state.snapshot.command_transcript.push(payload.line);
        break;
      case "QuitRequested":
        state.connected = false;
        state.busy = false;
        setNotice("Quit requested.", "ok");
        break;
      default:
        break;
    }
  }
}

function eventVariant(event) {
  if (typeof event === "string") {
    return { variant: event, payload: {} };
  }
  if (!event || typeof event !== "object") {
    return { variant: "", payload: {} };
  }
  const variant = Object.keys(event)[0] || "";
  return { variant, payload: event[variant] || {} };
}

function normalizeSnapshot(snapshot) {
  return {
    command_transcript: Array.isArray(snapshot?.command_transcript)
      ? snapshot.command_transcript
      : [],
    sessions: Array.isArray(snapshot?.sessions) ? snapshot.sessions : [],
  };
}

function ensureSelection() {
  if (
    state.selectedAgentId &&
    state.snapshot.sessions.some((session) => idKey(session.id) === state.selectedAgentId)
  ) {
    return;
  }

  state.selectedAgentId = state.snapshot.sessions[0]
    ? idKey(state.snapshot.sessions[0].id)
    : null;
}

function selectCommandSurface() {
  state.selectedAgentId = null;
  render();
}

function selectAgent(agentId) {
  state.selectedAgentId = agentId;
  render();
}

function upsertSession(session) {
  const key = idKey(session.id);
  const index = state.snapshot.sessions.findIndex((item) => idKey(item.id) === key);
  if (index === -1) {
    state.snapshot.sessions.push(session);
  } else {
    state.snapshot.sessions[index] = session;
  }
}

function upsertSessionStatus(payload) {
  const key = idKey(payload.agent_id);
  let session = state.snapshot.sessions.find((item) => idKey(item.id) === key);
  if (!session) {
    session = {
      id: payload.agent_id,
      kind: payload.kind,
      title: payload.title,
      feature: payload.feature,
      lines: [],
      loading: payload.loading,
      completion: payload.completion,
      token_usage: null,
    };
    state.snapshot.sessions.push(session);
    if (!state.selectedAgentId) {
      state.selectedAgentId = key;
    }
    return;
  }

  session.kind = payload.kind;
  session.title = payload.title;
  session.feature = payload.feature;
  session.loading = payload.loading;
  session.completion = payload.completion;
}

function updateSessionUsage(agentId, tokenUsage) {
  const session = findSession(agentId);
  if (session) {
    session.token_usage = tokenUsage;
  }
}

function appendSessionLine(agentId, line) {
  const session = findSession(agentId);
  if (session) {
    session.lines.push(line);
  }
}

function findSession(agentId) {
  const key = idKey(agentId);
  return state.snapshot.sessions.find((session) => idKey(session.id) === key);
}

function render() {
  renderStatus();
  renderSessions();
  renderThread();
  renderControls();
}

function renderStatus() {
  dom.connectionStatus.textContent = state.connected ? "Connected" : "Offline";
  dom.connectionStatus.className = `status-pill ${state.connected ? "ok" : "muted"}`;
  dom.busyStatus.textContent = state.busy ? "Busy" : "Idle";
  dom.busyStatus.className = `status-pill ${state.busy ? "warn" : "muted"}`;
}

function renderSessions() {
  dom.commandSurface.classList.toggle("selected", !state.selectedAgentId);
  dom.sessionList.replaceChildren(
    ...state.snapshot.sessions.map((session) => {
      const key = idKey(session.id);
      const button = document.createElement("button");
      button.type = "button";
      button.className = "session-row";
      button.classList.toggle("selected", key === state.selectedAgentId);
      button.addEventListener("click", () => selectAgent(key));

      const title = document.createElement("span");
      title.className = "session-title";
      title.textContent = session.title || session.feature || key;

      const meta = document.createElement("span");
      meta.className = "session-meta";
      meta.textContent = sessionStatusText(session);

      button.append(title, meta);
      return button;
    }),
  );
}

function renderThread() {
  const session = state.selectedAgentId ? findSession(state.selectedAgentId) : null;
  if (session) {
    dom.threadTitle.textContent = session.title || session.feature || idKey(session.id);
    dom.threadMeta.textContent = `${idKey(session.id)} - ${kindText(session.kind)}`;
    dom.threadStatus.textContent = sessionStatusText(session);
    dom.decisionBar.hidden = session.completion !== "NeedsDecision";
    renderLines(session.lines, "agent");
  } else {
    dom.threadTitle.textContent = "Command";
    dom.threadMeta.textContent = "work-leaf command surface";
    dom.threadStatus.textContent = `${state.snapshot.sessions.length} sessions`;
    dom.decisionBar.hidden = true;
    renderLines(state.snapshot.command_transcript, "command");
  }
}

function renderLines(lines, surface) {
  const fragment = document.createDocumentFragment();
  const safeLines = Array.isArray(lines) ? lines : [];
  if (safeLines.length === 0) {
    const empty = document.createElement("p");
    empty.className = "empty-state";
    empty.textContent = surface === "agent" ? "No agent output yet." : "No command output yet.";
    fragment.append(empty);
  } else {
    for (const line of safeLines) {
      const item = document.createElement("article");
      item.className = `line ${lineTone(line)}`;
      item.textContent = line;
      fragment.append(item);
    }
  }

  const shouldStickToBottom =
    dom.transcript.scrollTop + dom.transcript.clientHeight >= dom.transcript.scrollHeight - 24;
  dom.transcript.replaceChildren(fragment);
  if (shouldStickToBottom) {
    dom.transcript.scrollTop = dom.transcript.scrollHeight;
  }
}

function renderControls() {
  const hasAgent = Boolean(state.selectedAgentId);
  dom.refreshButton.disabled = !state.apiBase;
  dom.shutdownButton.disabled = !state.connected;
  dom.launchButton.disabled = !state.connected;
  dom.commandButton.disabled = !state.connected;
  dom.reviewButton.disabled = !state.connected;
  dom.linearizeButton.disabled = !state.connected;
  dom.sendButton.disabled = !state.connected;
  dom.messageInput.disabled = !state.connected;
  dom.interruptButton.disabled = !state.connected || !hasAgent;
  dom.messageInput.placeholder = hasAgent
    ? "Message selected agent"
    : "Message command agent";
}

function setNotice(message, tone) {
  dom.notice.textContent = message;
  dom.notice.className = `notice ${tone || ""}`;
}

function sessionStatusText(session) {
  if (session.loading === "Launching") {
    return "Launching";
  }
  if (session.loading === "WaitingForReply") {
    return "Waiting";
  }
  if (session.completion === "NeedsDecision") {
    return "Needs decision";
  }
  if (session.completion === "Closed") {
    return "Closed";
  }
  if (session.token_usage) {
    return tokenUsageText(session.token_usage);
  }
  return session.feature || "Ready";
}

function tokenUsageText(tokenUsage) {
  if (!tokenUsage || typeof tokenUsage !== "object") {
    return "Ready";
  }

  const entries = Object.entries(tokenUsage)
    .filter(([, value]) => typeof value === "number")
    .map(([key, value]) => `${key.replaceAll("_", " ")} ${value}`);

  return entries.length > 0 ? entries.join(" - ") : "Ready";
}

function lineTone(line) {
  if (line.startsWith("user:") || line.startsWith("work-leaf>")) {
    return "from-user";
  }
  if (line.includes(" error:") || line.startsWith("error:")) {
    return "from-error";
  }
  if (line.startsWith("command-agent:") || line.includes("launching")) {
    return "from-status";
  }
  return "from-agent";
}

function kindText(kind) {
  if (typeof kind === "string") {
    return kind;
  }
  if (kind && typeof kind === "object") {
    if (kind.External) {
      return kind.External;
    }
    return Object.keys(kind)[0] || "agent";
  }
  return "agent";
}

function idKey(agentId) {
  if (typeof agentId === "string") {
    return agentId;
  }
  if (agentId && typeof agentId === "object" && typeof agentId.id === "string") {
    return agentId.id;
  }
  return String(agentId);
}

function trimTrailingSlash(value) {
  return value.trim().replace(/\/+$/, "");
}

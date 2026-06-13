local process = require("leaf_blame.process")

local M = {}

local function trim(text)
  return (text or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

local function snapshot_url(config)
  local base_url = config.orchestrator_url or vim.env.WORK_LEAF_ORCHESTRATOR_URL
  if not base_url or base_url == "" then
    return nil, "WORK_LEAF_ORCHESTRATOR_URL is not set"
  end
  return base_url:gsub("/+$", "") .. "/snapshot", nil
end

function M.snapshot(config)
  local url, url_err = snapshot_url(config or {})
  if not url then
    return nil, url_err
  end

  local output, err = process.system({ "curl", "-fsS", url })
  if not output then
    return nil, "daemon snapshot request failed: " .. trim(err)
  end

  local ok, decoded = pcall(vim.json.decode, output)
  if not ok then
    return nil, "daemon snapshot response is not valid JSON"
  end

  return decoded, nil
end

function M.find(agent_id, config)
  local snapshot, err = M.snapshot(config or {})
  if not snapshot then
    return nil, err
  end

  for _, item in ipairs(snapshot.sessions or {}) do
    if item.id == agent_id then
      return item, nil
    end
  end

  return nil, "session " .. agent_id .. " is not present in the running daemon snapshot"
end

return M


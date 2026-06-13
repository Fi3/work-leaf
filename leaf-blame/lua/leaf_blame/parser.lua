local M = {}

local function trim(value)
  return (value or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

function M.commit_metadata(body)
  local metadata = {}

  for line in tostring(body or ""):gmatch("[^\r\n]+") do
    local key, value = line:match("^%s*([%w%-]+):%s*(.-)%s*$")
    if key == "Agent-ID" then
      metadata.agent_id = trim(value)
    elseif key == "Feature" then
      metadata.feature = trim(value)
    elseif key == "Reason" then
      metadata.reason = trim(value)
    elseif key == "Context" then
      metadata.context = trim(value)
    end
  end

  if metadata.agent_id == "" then
    metadata.agent_id = nil
  end

  return metadata
end

function M.parse_blame_porcelain(text)
  local entries = {}
  local current_hash = nil
  local current_final_line = nil

  for line in (tostring(text or "") .. "\n"):gmatch("(.-)\n") do
    local hash, _original_line, final_line = line:match("^([0-9a-fA-F]+)%s+(%d+)%s+(%d+)%s*%d*$")
    if hash then
      current_hash = hash
      current_final_line = tonumber(final_line)
    elseif line:sub(1, 1) == "\t" and current_hash and current_final_line then
      entries[current_final_line] = {
        hash = current_hash,
      }
      current_hash = nil
      current_final_line = nil
    end
  end

  return entries
end

function M.relative_path(root, path)
  root = tostring(root or ""):gsub("/+$", "")
  path = tostring(path or "")

  if root == "" then
    return path
  end

  local prefix = root .. "/"
  if path:sub(1, #prefix) == prefix then
    return path:sub(#prefix + 1)
  end

  if path == root then
    return "."
  end

  return path
end

return M


local parser = require("leaf_blame.parser")
local process = require("leaf_blame.process")
local session = require("leaf_blame.session")

local M = {}

local namespace = vim.api.nvim_create_namespace("leaf-blame")

local state = {
  enabled = {},
  line_entries = {},
  commit_metadata = {},
}

local defaults = {
  orchestrator_url = nil,
  virtual_text_prefix = "Leaf ",
  highlight = "Comment",
  map_mouse = false,
}

local config = vim.deepcopy(defaults)

local function notify(message, level)
  vim.notify(message, level or vim.log.levels.INFO, { title = "leaf-blame" })
end

local function normalize_bufnr(bufnr)
  if bufnr == nil or bufnr == 0 then
    return vim.api.nvim_get_current_buf()
  end
  return bufnr
end

local function trim(text)
  return (text or ""):gsub("^%s+", ""):gsub("%s+$", "")
end

local function git_root(path)
  local dir = vim.fs.dirname(path)
  local output, err = process.system({ "git", "-C", dir, "rev-parse", "--show-toplevel" })
  if not output then
    return nil, trim(err)
  end
  return trim(output), nil
end

local function metadata_for_commit(root, hash)
  if hash:match("^0+$") then
    return {}
  end

  local cache_key = root .. "\n" .. hash
  if state.commit_metadata[cache_key] then
    return state.commit_metadata[cache_key]
  end

  local output = process.system({ "git", "-C", root, "show", "-s", "--format=%B", hash })
  local metadata = {}
  if output then
    metadata = parser.commit_metadata(output)
    metadata.hash = hash
    metadata.subject = output:match("([^\r\n]*)") or ""
  end

  state.commit_metadata[cache_key] = metadata
  return metadata
end

local function clear_buffer(bufnr)
  vim.api.nvim_buf_clear_namespace(bufnr, namespace, 0, -1)
end

local function render_buffer(bufnr, entries)
  clear_buffer(bufnr)
  local line_count = vim.api.nvim_buf_line_count(bufnr)

  for lnum, entry in pairs(entries) do
    if lnum >= 1 and lnum <= line_count and entry.agent_id then
      local label = config.virtual_text_prefix .. entry.agent_id
      if entry.feature and entry.feature ~= "" then
        label = label .. " " .. entry.feature
      end
      vim.api.nvim_buf_set_extmark(bufnr, namespace, lnum - 1, 0, {
        virt_text = { { label, config.highlight } },
        virt_text_pos = "eol",
        hl_mode = "combine",
      })
    end
  end
end

local function set_mouse_mapping(bufnr)
  if not config.map_mouse then
    return
  end

  vim.keymap.set("n", "<LeftMouse>", function()
    local mouse = vim.api.nvim_replace_termcodes("<LeftMouse>", true, false, true)
    vim.api.nvim_feedkeys(mouse, "n", false)
    vim.schedule(function()
      M.open_current({ silent = true })
    end)
  end, {
    buffer = bufnr,
    desc = "Open Work Leaf chat for blamed line",
  })
end

local function clear_mouse_mapping(bufnr)
  pcall(vim.keymap.del, "n", "<LeftMouse>", { buffer = bufnr })
end

function M.setup(opts)
  config = vim.tbl_deep_extend("force", config, opts or {})
end

function M.refresh(bufnr)
  bufnr = normalize_bufnr(bufnr)
  if not vim.api.nvim_buf_is_valid(bufnr) then
    return
  end

  local path = vim.api.nvim_buf_get_name(bufnr)
  if path == "" then
    return
  end

  local root, root_err = git_root(path)
  if not root then
    notify("git root not found: " .. root_err, vim.log.levels.WARN)
    return
  end

  local relative_path = parser.relative_path(root, path)
  local blame_output, blame_err = process.system({
    "git",
    "-C",
    root,
    "blame",
    "--line-porcelain",
    "--",
    relative_path,
  })
  if not blame_output then
    notify("git blame failed: " .. trim(blame_err), vim.log.levels.WARN)
    return
  end

  local blame_entries = parser.parse_blame_porcelain(blame_output)
  local leaf_entries = {}

  for lnum, entry in pairs(blame_entries) do
    local metadata = metadata_for_commit(root, entry.hash)
    if metadata.agent_id then
      leaf_entries[lnum] = vim.tbl_extend("force", entry, metadata)
    end
  end

  state.line_entries[bufnr] = leaf_entries
  render_buffer(bufnr, leaf_entries)
end

function M.enable(bufnr)
  bufnr = normalize_bufnr(bufnr)
  if state.enabled[bufnr] then
    M.refresh(bufnr)
    return
  end

  state.enabled[bufnr] = true
  local group_name = "leaf_blame_" .. bufnr
  local group = vim.api.nvim_create_augroup(group_name, { clear = true })
  vim.api.nvim_create_autocmd({ "BufWritePost" }, {
    group = group,
    buffer = bufnr,
    callback = function()
      M.refresh(bufnr)
    end,
  })
  vim.api.nvim_create_autocmd({ "BufDelete", "BufWipeout" }, {
    group = group,
    buffer = bufnr,
    callback = function()
      state.enabled[bufnr] = nil
      state.line_entries[bufnr] = nil
    end,
  })

  set_mouse_mapping(bufnr)
  M.refresh(bufnr)
end

function M.disable(bufnr)
  bufnr = normalize_bufnr(bufnr)
  state.enabled[bufnr] = nil
  state.line_entries[bufnr] = nil
  clear_buffer(bufnr)
  clear_mouse_mapping(bufnr)
  pcall(vim.api.nvim_del_augroup_by_name, "leaf_blame_" .. bufnr)
end

function M.toggle(bufnr)
  bufnr = normalize_bufnr(bufnr)
  if state.enabled[bufnr] then
    M.disable(bufnr)
  else
    M.enable(bufnr)
  end
end

local function open_scratch(title, lines)
  local buf = vim.api.nvim_create_buf(false, true)
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].filetype = "markdown"
  vim.api.nvim_buf_set_name(buf, title)
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.cmd("botright split")
  vim.api.nvim_win_set_buf(0, buf)
end

function M.open_current(opts)
  opts = opts or {}
  local bufnr = vim.api.nvim_get_current_buf()
  local lnum = vim.api.nvim_win_get_cursor(0)[1]
  local entries = state.line_entries[bufnr]
  if not entries then
    M.refresh(bufnr)
    entries = state.line_entries[bufnr]
  end

  local entry = entries and entries[lnum]
  if not entry or not entry.agent_id then
    if not opts.silent then
      notify("current line is not from a Work Leaf commit", vim.log.levels.INFO)
    end
    return
  end

  local found_session, err = session.find(entry.agent_id, config)
  if not found_session then
    open_scratch("leaf-blame://" .. entry.agent_id, {
      "# Work Leaf " .. entry.agent_id,
      "",
      "Commit: " .. entry.hash,
      "Feature: " .. (entry.feature or ""),
      "Reason: " .. (entry.reason or ""),
      "",
      err,
    })
    return
  end

  local lines = {
    "# Work Leaf " .. found_session.id,
    "",
    "Title: " .. (found_session.title or ""),
    "Feature: " .. (found_session.feature or ""),
    "",
  }
  vim.list_extend(lines, found_session.lines or {})
  open_scratch("leaf-blame://" .. found_session.id, lines)
end

return M


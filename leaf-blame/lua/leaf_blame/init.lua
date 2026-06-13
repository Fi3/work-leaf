local blame = require("leaf_blame.blame")

local M = {}

local commands_created = false

local function create_commands()
  if commands_created then
    return
  end
  commands_created = true

  vim.api.nvim_create_user_command("LeafBlameEnable", function()
    blame.enable(0)
  end, {})

  vim.api.nvim_create_user_command("LeafBlameDisable", function()
    blame.disable(0)
  end, {})

  vim.api.nvim_create_user_command("LeafBlameToggle", function()
    blame.toggle(0)
  end, {})

  vim.api.nvim_create_user_command("LeafBlameRefresh", function()
    blame.refresh(0)
  end, {})

  vim.api.nvim_create_user_command("LeafBlameOpen", function()
    blame.open_current()
  end, {})

  vim.keymap.set("n", "<Plug>(LeafBlameOpen)", function()
    blame.open_current()
  end, { desc = "Open Work Leaf chat for blamed line" })
end

function M.setup(opts)
  blame.setup(opts or {})
  create_commands()
end

function M.enable(bufnr)
  blame.enable(bufnr or 0)
end

function M.disable(bufnr)
  blame.disable(bufnr or 0)
end

function M.toggle(bufnr)
  blame.toggle(bufnr or 0)
end

function M.refresh(bufnr)
  blame.refresh(bufnr or 0)
end

function M.open_current()
  blame.open_current()
end

return M


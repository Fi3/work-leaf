local M = {}

function M.system(cmd, opts)
  opts = opts or {}

  if vim.system then
    local result = vim.system(cmd, vim.tbl_extend("force", { text = true }, opts)):wait()
    if result.code == 0 then
      return result.stdout or "", nil
    end
    return nil, result.stderr or result.stdout or ""
  end

  local output = vim.fn.system(cmd)
  if vim.v.shell_error == 0 then
    return output, nil
  end
  return nil, output
end

return M


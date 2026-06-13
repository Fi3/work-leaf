if vim.g.loaded_leaf_blame == 1 then
  return
end

vim.g.loaded_leaf_blame = 1

require("leaf_blame").setup()


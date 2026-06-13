# leaf-blame.nvim

Neovim plugin for showing Work Leaf provenance on blamed lines.

The plugin reads normal git blame output for the current buffer, inspects each blamed commit body,
and treats commits with Work Leaf metadata as Leaf-authored lines. Work Leaf provisional patch
commits include metadata such as:

```text
Agent-ID: user-1
Feature: parser
Reason: return parsed value
```

When a blamed line comes from one of those commits, `leaf-blame.nvim` adds virtual text with the
Leaf session id. Opening that line queries the Work Leaf daemon snapshot and displays the matching
active chat transcript.

## Local Installation

With `lazy.nvim`, load the plugin directly from this repository:

```lua
{
  dir = "/home/user/src/work-leaf/leaf-blame",
  name = "leaf-blame.nvim",
  config = function()
    require("leaf_blame").setup({
      orchestrator_url = vim.env.WORK_LEAF_ORCHESTRATOR_URL,
    })
  end,
}
```

Start Work Leaf so the daemon URL is available to Neovim:

```sh
cargo build --bins
WORK_LEAF_ORCHESTRATOR_URL=http://127.0.0.1:7878 nvim
```

Or start the daemon explicitly:

```sh
target/debug/work-leaf-orchestrator --listen 127.0.0.1:7878
```

Then configure:

```lua
require("leaf_blame").setup({
  orchestrator_url = "http://127.0.0.1:7878",
})
```

## Commands

- `:LeafBlameEnable` enables Leaf blame virtual text for the current buffer.
- `:LeafBlameDisable` disables it for the current buffer.
- `:LeafBlameToggle` toggles it for the current buffer.
- `:LeafBlameRefresh` recomputes blame for the current buffer.
- `:LeafBlameOpen` opens the Work Leaf chat history for the current cursor line.

The plugin also defines `<Plug>(LeafBlameOpen)` so users can choose their own mapping:

```lua
vim.keymap.set("n", "<leader>lh", "<Plug>(LeafBlameOpen)")
```

Mouse opening is optional because it overrides the buffer-local left-click mapping while enabled:

```lua
require("leaf_blame").setup({
  map_mouse = true,
})
```

## Limits

The git provenance is persistent because it comes from commit metadata. Chat history comes from the
running daemon's `GET /snapshot` response, so old sessions are available only while the daemon still
has them in memory.

## Tests

Parser tests run without Neovim:

```sh
lua leaf-blame/tests/parser_spec.lua
```


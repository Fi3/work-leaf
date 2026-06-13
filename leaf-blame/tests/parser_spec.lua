package.path = table.concat({
  "leaf-blame/lua/?.lua",
  "leaf-blame/lua/?/init.lua",
  package.path,
}, ";")

local parser = require("leaf_blame.parser")

local function assert_equal(actual, expected)
  if actual ~= expected then
    error(string.format("expected %q, got %q", tostring(expected), tostring(actual)), 2)
  end
end

local metadata = parser.commit_metadata([[
UPDATE apply parser patch from user-1

Agent-ID: user-1
Feature: parser
Reason: return parsed value
Context: touched src/lib.rs
]])

assert_equal(metadata.agent_id, "user-1")
assert_equal(metadata.feature, "parser")
assert_equal(metadata.reason, "return parsed value")
assert_equal(metadata.context, "touched src/lib.rs")

local no_metadata = parser.commit_metadata([[
UPDATE normal human commit

Reviewed-by: human
]])

assert_equal(no_metadata.agent_id, nil)

local blame = parser.parse_blame_porcelain([[
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa 1 1 1
author A
summary first
filename src/lib.rs
	line one
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb 2 2 1
author B
summary second
filename src/lib.rs
	line two
]])

assert_equal(blame[1].hash, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
assert_equal(blame[2].hash, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")

assert_equal(
  parser.relative_path("/repo/work-leaf", "/repo/work-leaf/src/lib.rs"),
  "src/lib.rs"
)
assert_equal(parser.relative_path("/repo/work-leaf", "/other/file"), "/other/file")

print("leaf-blame parser tests passed")


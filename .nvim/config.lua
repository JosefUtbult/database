
local ctx = require("exrc").init()
local dap = require("dap")
local lspconfig = require("lspconfig")
local rust_config = require("utils.lsp_configs")["rust_analyzer"]

-- Disable only cwd old files
vim.g.telescope_cwd_only = false

rust_config.settings["rust-analyzer"].workspace = rust_config.settings["rust-analyzer"].workspace or {}

rust_config.settings["rust-analyzer"].cargo = {
	allFeatures = true,
	loadOutDirsFromCheck = true,
	noDefaultFeatures = false,
	cfgs = { "test" },
}

lspconfig.rust_analyzer.setup(rust_config)

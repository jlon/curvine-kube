# Makefile for curvine-kube
# This is a convenience wrapper around cargo xtask

.PHONY: help build test dist install ci format clippy clean

help: ## Show this help message
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project (debug mode)
	cargo xtask build

release: ## Build the project (release mode)
	cargo xtask build --release

test: ## Run all tests
	cargo xtask test

test-integration: ## Run integration tests only
	cargo xtask test --integration

dist: ## Create distribution package
	cargo xtask dist

install: ## Install to /usr/local
	cargo xtask install

install-to: ## Install to custom prefix (use PREFIX=/path)
	cargo xtask install --prefix $(PREFIX)

ci: ## Run CI checks (format, clippy, test)
	cargo xtask ci

format: ## Format code
	cargo xtask format

format-check: ## Check code formatting
	cargo xtask format --check

clippy: ## Run clippy lints
	cargo xtask clippy

clean: ## Clean build artifacts
	cargo clean
	rm -rf dist/
	rm -f *.tar.gz

.DEFAULT_GOAL := help

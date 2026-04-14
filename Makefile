BINARY  = solarust
RELEASE = target/release/$(BINARY)

# Install prefix — override with: make install PREFIX=/usr/local
# Default: ~/.local  (no sudo needed, works on Linux & macOS)
PREFIX ?= $(HOME)/.local
BINDIR  = $(PREFIX)/bin

.PHONY: all build install uninstall clean

all: build

setup:
	cp scripts/commit-msg .git/hooks/commit-msg
	chmod +x .git/hooks/commit-msg

build:
	cargo build --release

install: build
	@mkdir -p $(BINDIR)
	@install -m 755 $(RELEASE) $(BINDIR)/$(BINARY)
	@echo "Installed $(BINARY) → $(BINDIR)/$(BINARY)"
	@echo ""
	@if ! echo "$$PATH" | grep -q "$(BINDIR)"; then \
		echo "  Note: $(BINDIR) is not in your PATH."; \
		echo "  Add this to your shell config (~/.bashrc, ~/.zshrc, …):"; \
		echo "    export PATH=\"$(BINDIR):\$$PATH\""; \
	fi

uninstall:
	@rm -f $(BINDIR)/$(BINARY)
	@echo "Removed $(BINDIR)/$(BINARY)"

clean:
	cargo clean

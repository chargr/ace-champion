PREFIX ?= ~/.local

.PHONY: install build release

.DEFAULT_GOAL := build

BINS := target/release/ace-champ
SOURCES := src/*.rs Cargo.toml Cargo.lock

build: $(BINS)

install: build
	install -d $(PREFIX)/bin
	for b in $(BINS); do \
		install -s -m 755 $$b $(PREFIX)/bin/; \
	done

$(BINS): $(SOURCES)
	cargo build --release

clean:
	cargo clean --release --package ace-champion

distclean:
	cargo clean

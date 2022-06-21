NAME        := prompt-bot

prefix      ?= /usr/local
exec_prefix ?= $(prefix)
bindir      ?= $(exec_prefix)/bin

bindestdir  := $(DESTDIR)$(bindir)

all: build

build:
	cargo build --locked --release --bins

cargo-install:
	cargo install --locked --path "."

cargo-uninstall:
	cargo uninstall --locked $(NAME)

installdirs:
	install -d $(bindestdir)/

install: installdirs
	install ./target/release/$(NAME) $(bindestdir)/

uninstall:
	rm -f $(bindestdir)/$(NAME)

test:
	cargo test

clean:
	cargo clean

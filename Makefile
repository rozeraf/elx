.PHONY: all build install uninstall clean changelog

all: build

build:
	cargo build --release

install: build
	install -d ~/.local/bin
	install -Dm755 target/release/elx ~/.local/bin/elx
	@echo "Installed to ~/.local/bin/elx"

uninstall:
	rm -f ~/.local/bin/elx

clean:
	cargo clean

# TAG=vX.Y.Z labels unreleased commits with the upcoming version.
changelog:
	git-cliff $(if $(TAG),--tag $(TAG)) -o CHANGELOG.md

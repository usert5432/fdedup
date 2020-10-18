BIN     = fdedup
CARGO   = cargo
VERSION = 0.1.0

SOURCES = $(shell find src/ -name '*.rs')

PREFIX  ?= /usr/local
DATADIR ?= $(PREFIX)/share
MANDIR  ?= $(DATADIR)/man
DISTDIR ?= fdedup-$(VERSION)

.PHONY: all clean dist install test

CARGOFLAGS ?= --release
BUILDDIR   ?= target/release

all: $(BUILDDIR)/$(BIN)

$(BUILDDIR)/$(BIN): $(SOURCES)
	$(CARGO) build $(CARGOFLAGS)

install: $(BUILDDIR)/$(BIN)
	mkdir -p ${DESTDIR}$(PREFIX)/bin
	cp -f $(BUILDDIR)/$(BIN) ${DESTDIR}$(PREFIX)/bin
	chmod 755 ${DESTDIR}$(PREFIX)/bin/$(BIN)
	mkdir -p ${DESTDIR}$(MANDIR)/man1
	sed "s/VERSION/$(VERSION)/g" man/fdedup.1 \
		> ${DESTDIR}$(MANDIR)/man1/fdedup.1
	chmod 644 ${DESTDIR}$(MANDIR)/man1/fdedup.1

test: $(SOURCES) $(wildcard tests/*.rs)
	$(CARGO) test $(CARGOFLAGS)

clean:
	$(CARGO) clean
	rm -f $(DISTDIR).tar.gz

dist:
	rm -f $(DISTDIR).tar.gz
	mkdir -p $(DISTDIR)
	cp -r src/ tests/ $(DISTDIR)/
	cp -f Makefile LICENSE README.rst $(DISTDIR)/
	cp -f Cargo.lock Cargo.toml .gitignore $(DISTDIR)/
	tar -c -f $(DISTDIR).tar --remove-files $(DISTDIR)
	gzip $(DISTDIR).tar


# OVM — Omni Virtual Machine Standalone Runtime
# Build: make
# Install: make install PREFIX=/usr/local

CC      ?= cc
CFLAGS  ?= -O2 -Wall -Wextra -std=c11
PREFIX  ?= /usr/local

# Platform-specific linker flags
ifeq ($(OS),Windows_NT)
LDFLAGS ?= -lm
else
LDFLAGS ?= -lm -lpthread
endif

TARGET = ovm
SRC    = ovm.c

.PHONY: all clean install test

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(TARGET) $(TARGET).exe

install: $(TARGET)
	install -d $(PREFIX)/bin
	install -m 755 $(TARGET) $(PREFIX)/bin/

test: $(TARGET)
	@echo "OVM built successfully"
	@./$(TARGET) --version

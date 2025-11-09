TARGET := riscv64gc-unknown-none-elf
MODE := debug
APP_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
BUILD_DIR := build
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64
PY := python3

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

BASE ?= 0
CHAPTER ?= 0
TEST ?= $(CHAPTER)

CHAPTER_NUM := $(shell echo $(CHAPTER) | sed 's/-debug//' | sed 's/[^0-9].*//')
ifeq ($(strip $(CHAPTER_NUM)),)
	CHAPTER_NUM := 0
endif
BIN_BEFORE_FS := $(shell [ $(CHAPTER_NUM) -lt 6 ] 2>/dev/null && echo 1 || echo 0)
TEST_NUM := $(shell echo $(TEST) | sed 's/-debug//')

ifeq ($(TEST_NUM), 0) # No test, deprecated, previously used in v3
	APPS :=  $(filter-out $(wildcard $(APP_DIR)/ch*.rs), $(wildcard $(APP_DIR)/*.rs))
else ifeq ($(TEST_NUM), 1) # All test
	APPS :=  $(wildcard $(APP_DIR)/ch*.rs)
else
	TESTS := $(shell seq $(BASE) $(TEST_NUM))
	ifeq ($(BASE), 0) # Normal tests only
		APPS := $(foreach T, $(TESTS), $(wildcard $(APP_DIR)/ch$(T)_*.rs))
	else ifeq ($(BASE), 1) # Basic tests only
		APPS := $(foreach T, $(TESTS), $(wildcard $(APP_DIR)/ch$(T)b_*.rs))
	else # Basic and normal
		APPS := $(foreach T, $(TESTS), $(wildcard $(APP_DIR)/ch$(T)*.rs))
	endif
endif

ELFS := $(patsubst $(APP_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BIN_OBJCMDS := $(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf)); )
BIN_COPYCMDS := $(foreach t, $(ELFS), cp $(t).bin $(BUILD_DIR)/bin/;)
ifeq ($(BIN_BEFORE_FS),0)
BIN_OBJCMDS :=
BIN_COPYCMDS :=
endif

binary:
	@printf '%s\n' $(ELFS)
	@if [ ${CHAPTER_NUM} -gt 3 ]; then \
		cargo build $(MODE_ARG) ;\
	else \
		CHAPTER=$(CHAPTER_NUM) python3 build.py ;\
	fi
	@$(if $(BIN_OBJCMDS),$(BIN_OBJCMDS),:)
	@$(foreach elf, $(ELFS), cp $(elf) $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.elf, $(elf));)

disasm:
	@$(foreach elf, $(ELFS), \
		$(OBJDUMP) $(elf) -S > $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.asm, $(elf));)
	@$(foreach t, $(ELFS), cp $(t).asm $(BUILD_DIR)/asm/;)

pre:
	@mkdir -p $(BUILD_DIR)/bin/
	@mkdir -p $(BUILD_DIR)/elf/
	@mkdir -p $(BUILD_DIR)/app/
	@mkdir -p $(BUILD_DIR)/asm/
	@$(foreach t, $(APPS), cp $(t) $(BUILD_DIR)/app/;)

build: clean pre binary
	@$(if $(BIN_COPYCMDS),$(BIN_COPYCMDS),:)
	@$(foreach t, $(ELFS), cp $(t).elf $(BUILD_DIR)/elf/;)

clean:
	@cargo clean
	@rm -rf $(BUILD_DIR)

all: build

.PHONY: elf binary build clean all

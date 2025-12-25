TARGET := skernel-x86
TARGET_ELF := ../../target/x86_64-unknown-none/release/$(TARGET)

all: $(TARGET) FORCE

$(TARGET): $(TARGET_ELF)
	@rust-objcopy --binary-architecture=x86_64 --strip-all -O binary $< $@

$(TARGET_ELF):
	@cargo build -p $(TARGET) --target x86_64-unknown-none --release

clean:
	@rm -rf ./$(TARGET)
	@cargo clean -p $(TARGET) --target x86_64-unknown-none --release

FORCE:

.PHONY: FORCE

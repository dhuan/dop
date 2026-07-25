build:
	cargo build --profile release

test:
	sh scripts/test.sh

.PHONY: test

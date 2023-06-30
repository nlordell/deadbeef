.PHONY: dist
dist: lib/pkg
	deno run --allow-all bundle.ts

.PHONY: lib/pkg
lib/pkg:
	wasm-pack build --target web --release --no-pack \
		--out-name deadbeef --out-dir lib/pkg

.PHONY: test
test: dist
	deno test test/index.ts

.PHONY: host
host: dist
	python3 -m http.server

.PHONY: clean
clean:
	rm -rf lib/pkg

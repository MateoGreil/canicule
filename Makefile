RUST ?= cargo
NPM ?= npm

.PHONY: dev build test

dev:
	cd frontend && npx concurrently -k "cd .. && $(RUST) run -p canicule" "$(NPM) run dev"

build:
	$(NPM) --prefix frontend run build
	@set -e; \
		backup=$$(mktemp); \
		cp proxy/static/index.html "$$backup"; \
		trap 'cp "$$backup" proxy/static/index.html; rm "$$backup"' EXIT; \
		rm -rf proxy/static/assets; \
		mkdir -p proxy/static/assets; \
		cp frontend/dist/index.html proxy/static/index.html; \
		cp -a frontend/dist/assets/. proxy/static/assets/; \
		$(RUST) build --release -p canicule

test:
	$(RUST) test -p canicule
	$(NPM) --prefix frontend test
	$(NPM) --prefix frontend run check
	$(NPM) --prefix frontend run build
	$(NPM) --prefix frontend run test:e2e

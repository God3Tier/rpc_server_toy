# Convenience wrapper around the docker compose workflow.
# Layout assumed: docker-compose.yml, mock/, client/, server/, zig-client/
# all at the repo root (no docker/ parent folder).

.PHONY: build build-client build-server rebuild up down server logs \
        test test-ctypes clean ps

# Build every image.
build:
	docker compose build

build-client:
	docker compose build client mock

build-server:
	docker compose build server

build-proxy: 
	docker compose build proxy-server

# Force a clean rebuild, bypassing layer cache -- useful when a Dockerfile
# change (e.g. an apt-get install line) doesn't seem to take effect.
rebuild:
	docker compose build --no-cache

# Start the server in the background and leave it running.
up: all-server

all-server:
	docker compose up -d server proxy-server

server:
	docker compose up -d server 

proxy-server:
	docker compose up -d proxy-server
	
# Show server logs, following.
logs:
	docker compose logs -f server proxy-server

proxy-logs: 
	docker compose logs -f proxy-server

server-logs: 
	docker compose logs -f server proxy-server

# ctypes-based test that calls the compiled mylib.so directly, exercising
# the real client code path. Uses tee so output is visible live AND saved.
test-ctypes:
	docker compose run --rm client python3 ctypes_test.py 2>&1 | tee output.log

#
reset-all-server:
	docker compose build --no-cache server
	docker compose up -d --force-recreate server
	docker compose build --no-cache proxy-server
	docker compose up -d --force-recreate proxy-server
	
reset-server:
	docker compose build --no-cache server
	docker compose up -d --force-recreate server

reset-proxy-server:
	docker compose build --no-cache server
	docker compose up -d --force-recreate server
	docker compose build --no-cache proxy-server
	docker compose up -d --force-recreate proxy-server

# Stop and remove all containers, networks (keeps built images).
down:
	docker compose down --remove-orphans

# Full teardown: containers, networks, and volumes.
clean:
	docker compose down --remove-orphans -v

# Quick status check.
ps:
	docker compose ps

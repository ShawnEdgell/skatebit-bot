# 🔎 ModFinder - The Skatebit Discord Bot

The official Discord bot for Skatebit, providing fast, up-to-date access to Skater XL mods and maps directly within your Discord server. ModFinder ensures you always have the latest information by leveraging Skatebit's custom Go API (which uses Redis) and Mod.io.

## ✨ Core Features

- **Search Maps:** Use `/map` with autocomplete to find Skater XL maps. Displays details like author, summary, image, and download link, sourced from the Skatebit API.
- **Search Versioned Mods:** Use `/mod` for specific game version script mods (uses a separate, community-maintained data source).
- **Mod List Link:** `/modlist` provides a quick link to the community mod list website.
- **Data Freshness:** Map data is kept up-to-date by the backend Go API's event-driven polling of Mod.io and Redis caching.

## 🚀 Key Technologies

- Rust (Serenity & Poise framework)
- Tokio (asynchronous runtime)
- Reqwest (for HTTP requests)
- **Redis** (as the primary data source for maps/scripts, accessed via the Go API)
- Docker & Docker Compose

## Running Locally (Docker Compose Recommended)

The bot relies on the `modio-api-go` service. Using Docker Compose is the easiest way to run both locally.

**Prerequisites:**

- Docker Desktop (or Docker Engine)
- Git
- `.env` files for both `skatebit-bot` and `modio-api-go` projects.

**Steps:**

1.  **Clone Repositories:** Ensure you have both `modio-api-go` and `skatebit-bot` cloned.
2.  **Build Images:**
    ```bash
    # In modio-api-go directory
    docker build -t modio-api-go-local .
    # In skatebit-bot directory
    docker build -t skatebit-bot-local .
    ```
3.  **Create `docker-compose.local.yml`:**
    In a common parent directory (or your bot's directory), create a `docker-compose.local.yml`:

    ```yaml
    version: "3.8"
    services:
      local_redis:
        image: redis:alpine
        container_name: test_local_redis
        ports:
          - "6379:6379"
        volumes:
          - redis_data_local:/data # Optional: for Redis data persistence locally

      modio_api_local:
        image: modio-api-go-local # Use the image you built
        container_name: test_modio_api_for_bot
        restart: unless-stopped
        env_file:
          - ./modio-api-go/.env # Adjust path to your Go API's .env file
        depends_on:
          local_redis:
            condition: service_started # Wait for Redis to start
        environment:
          # Ensure REDIS_ADDR points to the Redis service name in this compose file
          - REDIS_ADDR=local_redis:6379
        # Add port mapping if you need to access Go API directly from host
        # ports:
        #   - "8001:8000" # Host:Container (Go API's internal port)
        healthcheck:
          test: ["CMD", "curl", "-f", "http://localhost:8000/health"] # Go API's internal port
          interval: 15s
          timeout: 5s
          retries: 5
          start_period: 20s

      discord_bot_local:
        image: skatebit-bot-local # Use the image you built
        container_name: test_discord_bot
        restart: unless-stopped
        env_file:
          - ./skatebit-bot/.env # Adjust path to your Bot's .env file
        depends_on:
          modio_api_local:
            condition: service_healthy # Wait for Go API to be healthy
          local_redis: # Bot also needs Redis for its own direct queries
            condition: service_started
        environment:
          # Ensure REDIS_URL points to the Redis service name
          - REDIS_URL=redis://local_redis:6379
          # GO_MODIO_API_BASE_URL is not strictly needed if bot queries Redis directly for maps/scripts
          # but might be used by mod_utils for the vercel app.
          # If mod_utils needs to hit the Go API, use:
          # - GO_MODIO_API_BASE_URL=http://modio_api_local:8000

    volumes:
      redis_data_local:
    ```

4.  **Prepare `.env` files:**
    - `./modio-api-go/.env`: Needs `MODIO_API_KEY`, `PORT=8000` (internal Go API port).
    - `./skatebit-bot/.env`: Needs `DISCORD_TOKEN` (your test bot token), `REDIS_URL=redis://local_redis:6379`. `TEST_GUILD_ID` if using guild commands for testing.
5.  **Run:** `docker compose -f docker-compose.local.yml up --build` (from directory with compose file).
6.  Test bot in Discord. Stop with `Ctrl+C` then `docker compose -f docker-compose.local.yml down`.

## Key Environment Variables (for `skatebit-bot`)

(See `.env.example` for all)

- `DISCORD_TOKEN`: **Required** Discord bot token.
- `REDIS_URL`: **Required** URL for the Redis instance (e.g., `redis://local_redis:6379` in Docker Compose, `redis://127.0.0.1:6379` for local host Redis).
- `RUST_LOG`: Logging level (e.g., `info,skatebit_bot=debug`).
- `TEST_GUILD_ID`: (Optional) For registering commands to a test guild during development.

## Deployment

The bot and its dependent Go API (with Redis) are deployed as Docker containers, managed via Docker Compose on a VPS.

## Project Structure

- `src/`: Rust source code (main, lib, commands, types, mod_utils, scheduler).
- `Cargo.toml`: Project dependencies.
- `Dockerfile`: Builds the bot's Docker image.

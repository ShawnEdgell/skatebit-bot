# 🔎 ModFinder - The Skatebit Discord Bot

The official Discord bot for Skatebit, providing fast, up-to-date access to Skater XL mods and maps directly within your Discord server. ModFinder ensures you always have the latest information by leveraging Skatebit's custom API and Mod.io.

## ✨ Features

- **Browse Full Mod Lists:** Use `/modlist` to receive a complete list of mods for specific game versions of Skater XL, delivered directly to your DMs. (Note: This feature currently uses a separate data source for specific mod versions).
- **Search Maps:** Use `/map` to search for Skater XL maps. Features fast autocomplete suggestions and provides detailed map information, including author, summary, images, and download links, sourced from our new Skatebit API.
- **Search General Mods:** Use `/mod` (or `/modsearch`) to find other game modifications. (Note: This feature currently uses a separate data source for specific mod versions).
- **Always Up-to-Date:**
  - Map data is refreshed frequently via the Skatebit API, which itself caches data from Mod.io with a hybrid polling strategy for freshness.
  - Specific versioned mod lists are also updated on a schedule.
- **Easy Invite:** Global slash commands mean you can invite ModFinder to any server and start using its features.

## 🚀 Key Technologies

- Rust (with Serenity and Poise framework for Discord integration)
- Tokio (for asynchronous runtime)
- Reqwest (for HTTP requests to the Skatebit API and Mod.io)
- Tracing (for structured logging)
- **Skatebit Go API (`api.skatebit.app`)**: The primary source for map data.
- Docker & Docker Compose (for containerization and deployment)
- Caddy (as the reverse proxy for the Go API on the host VPS)

## Running for Local Development (Using Docker)

This is the recommended way to run the bot locally for development and testing, as it includes its dependency on the Mod.io Cache API (Go service).

### Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (or Docker Engine for Linux) installed.
- Git (for cloning repositories).
- A local `.env` file for `skatebit-bot` (see "Environment Variables" below).
- A local `.env` file for the `modio-api-go` project (containing `MODIO_API_KEY` and `PORT=8000`).
- Locally built Docker images: `modio-api-go-local` and `skatebit-bot-local`.

### Steps

1.  **Clone both repositories** (if you haven't already) into a common parent directory (e.g., `~/repos/`):
    ```bash
    git clone [https://github.com/ShawnEdgell/modio-api-go.git](https://github.com/ShawnEdgell/modio-api-go.git)
    git clone [https://github.com/ShawnEdgell/skatebit-bot.git](https://github.com/ShawnEdgell/skatebit-bot.git)
    ```
2.  **Build the local Docker images** for both services:

    ```bash
    # In your modio-api-go project directory
    docker build -t modio-api-go-local .

    # In your skatebit-bot project directory
    docker build -t skatebit-bot-local .
    ```

3.  **Create a `docker-compose.local.yml` file** in your parent `repos` directory (or a dedicated test environment directory) with content similar to this:

    ```yaml
    # docker-compose.local.yml
    services:
      modio_api_local:
        image: modio-api-go-local
        container_name: test_modio_api_for_bot
        env_file:
          - ./modio-api-go/.env # Path relative to this compose file
        healthcheck:
          test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
          interval: 10s
          timeout: 5s
          retries: 5
          start_period: 20s

      discord_bot_local:
        image: skatebit-bot-local
        container_name: test_discord_bot
        restart: unless-stopped
        env_file:
          - ./skatebit-bot/.env # Path relative to this compose file
        # Ensure GO_MODIO_API_BASE_URL in skatebit-bot/.env is http://modio_api_local:8000
        depends_on:
          modio_api_local:
            condition: service_healthy
    ```

4.  **Prepare `.env` files:**
    - `./modio-api-go/.env`: Must contain `MODIO_API_KEY` and `PORT=8000`.
    - `./skatebit-bot/.env`: Must contain `DISCORD_BOT_TOKEN` and `GO_MODIO_API_BASE_URL=http://modio_api_local:8000`. Also include `MODIO_API_KEY` if your `mod_utils` still hits Mod.io directly.
5.  **Run Docker Compose:**
    From the directory containing your `docker-compose.local.yml`:
    ```bash
    docker compose -f docker-compose.local.yml up
    ```
    (Add `-d` to run in detached mode. Add `--build` if you want to force image rebuilds).
6.  Observe logs and test bot functionality in Discord.
7.  To stop: `Ctrl+C` then `docker compose -f docker-compose.local.yml down`.

## Environment Variables (for `skatebit-bot`)

Create a `.env` file in the root of the `skatebit-bot` project (and ensure it's in `.gitignore`):

- `DISCORD_BOT_TOKEN`: **Required.** Your Discord bot token.
- `GO_MODIO_API_BASE_URL`: **Required.** The base URL for the self-hosted Go API.
  - For local Docker Compose testing: `http://modio_api_local:8000` (service name and internal port of the Go API container).
  - For VPS deployment: `http://modio_api_container:8000` (service name and internal port of the Go API container on VPS).
- `MODIO_API_KEY`: Required if your `mod_utils` (for slug-based mod fetching) still makes direct calls to Mod.io.
- `RUST_LOG`: Logging level (e.g., `info,skatebit_bot=debug`).
- `TEST_GUILD_ID`: (Optional, for development) If you want to register commands to a specific test guild for faster updates during development (requires code changes in `lib.rs` to use it).

## Deployment (VPS)

This bot is designed to be deployed as a Docker container on a VPS, managed by a central `docker-compose.yml` file (typically located in `~/projects/` on the VPS).

1.  The `modio-api-go` service (providing map data) runs as a separate container on the same Docker network.
2.  The bot container (`skatebit-bot-app` image) is configured to use the Go API via its Docker service name (e.g., `GO_MODIO_API_BASE_URL=http://modio_api_container:8000`).
3.  Environment variables (especially `DISCORD_BOT_TOKEN`) are supplied from a central `.env` file on the VPS.
4.  Caddy is not directly used by the bot unless it exposes an HTTP endpoint, but it serves the Go API the bot consumes.

## Project Structure (This Repository)

- `src/`: Contains all Rust source code.
  - `main.rs`: Bot entry point.
  - `lib.rs`: Core bot logic, framework setup, event handling.
  - `commands/`: Modules for different bot commands (ping, map, modlist, etc.).
  - `types.rs`: Struct definitions for data, context, errors.
  - `map_cache.rs`: Logic for fetching and caching map data (now from the Go API).
  - `mod_utils.rs`: Logic for fetching specific versioned mods (current separate source).
  - `scheduler.rs`: Handles periodic cache updates.
- `Cargo.toml`, `Cargo.lock`: Rust project and dependency management.
- `Dockerfile`: Instructions to build the Docker image for the bot.
- `.dockerignore`: Specifies files to exclude from the Docker build context.
- `.gitignore`: Specifies files for Git to ignore.
- `.env.example`: Template for required environment variables.
- `README.md`: This file.

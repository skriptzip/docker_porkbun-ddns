# <img src="https://porkbun.com/images/porkbun-logo-1200x1200.png" height="32" style="vertical-align:middle; margin-right:8px;"/> Porkbun DDNS

A lightweight Docker-based Dynamic DNS updater for Porkbun domains. This service continuously monitors your public IP address and automatically updates your Porkbun DNS records when changes are detected.

## Features

- **Automatic DNS Updates** - Detects IP changes and updates Porkbun DNS records
- **Multiple Records** - Update several subdomains across different domains at once
- **Docker Ready** - Pre-configured with Docker and Docker Compose
- **Configurable** - Customizable check intervals and IP detection sources
- **Persistent State** - Tracks the last known IP to avoid unnecessary API calls
- **Tiny** - Written in Rust, ships as a static binary in a `scratch` image (a few MB)

## Prerequisites

- Docker and Docker Compose
- Porkbun account with API credentials
- One or more domains registered with Porkbun, with API access enabled for each

## Setup

### 1. Get Porkbun API Credentials

1. Log in to your [Porkbun account](https://porkbun.com)
2. Navigate to **Account > API Access**
3. Enable API access and generate your API key and secret key
4. Note the credentials - you'll need them in the next step

### 2. Configure Environment Variables

Create a `.env` file in the project root directory with the following variables:

```env
PORKBUN_API_KEY=your_api_key_here
PORKBUN_SECRET_API_KEY=your_secret_api_key_here
HOSTS=local.example.com,dashboard.example.org,files.example.org
CHECK_INTERVAL=300
```

**Configuration Options:**

- `PORKBUN_API_KEY` - Your Porkbun API key (required)
- `PORKBUN_SECRET_API_KEY` - Your Porkbun API secret key (required)
- `HOSTS` - Comma separated list of records to update (required unless `DOMAIN`/`SUBDOMAIN` are set, see below)
- `DOMAIN` / `SUBDOMAIN` - Single record, only used when `HOSTS` is not set (use `@` as subdomain for the root domain)
- `CHECK_INTERVAL` - Time in seconds between IP checks (optional, default: `300`)
- `IP_CHECK_URL` - Custom IP detection service (optional, default: `https://api.ipify.org`)

### Multiple records and domains

`HOSTS` accepts full hostnames separated by commas (spaces or semicolons also work). The records can belong to different domains in the same Porkbun account:

```env
HOSTS=local.example.com,dashboard.example.org,files.example.org
```

- `example.com` (two labels) updates the root record of that domain.
- The registered domain is assumed to be the last two labels. For multi-part TLDs, write `sub@example.co.uk` (or `@example.co.uk` for the root).
- Each record is tracked separately in `data/last_ip.txt`. If one update fails, the others are still applied and only the failed one is retried at the next check.
- API access must be enabled for every domain in the Porkbun domain settings.

### 3. Run with Docker Compose

```bash
docker-compose up -d
```

This will:
- Build the Docker image
- Start the DDNS service in the background
- Automatically restart on failure
- Mount the `data/` directory for persistent IP tracking (it must be writable for UID 1001)

## Usage

### View Logs

```bash
docker-compose logs -f ddns
```

### Stop the Service

```bash
docker-compose down
```

### Update Configuration

1. Modify the `.env` file
2. Recreate the container so the new values are picked up (`restart` alone does not reload `.env`):
   ```bash
   docker-compose up -d
   ```

## Environment Variables Explained

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORKBUN_API_KEY` | Yes | - | Your Porkbun API key |
| `PORKBUN_SECRET_API_KEY` | Yes | - | Your Porkbun API secret key |
| `HOSTS` | Yes* | - | Comma separated hostnames to update, may span multiple domains |
| `DOMAIN` | Yes* | - | Single domain, alternative to `HOSTS` |
| `SUBDOMAIN` | Yes* | - | Single subdomain (`@` for root), alternative to `HOSTS` |
| `CHECK_INTERVAL` | No | 300 | Seconds between IP checks |
| `IP_CHECK_URL` | No | https://api.ipify.org | Service to check public IP |

\* Either `HOSTS`, or both `DOMAIN` and `SUBDOMAIN`, must be set.

## Development

```bash
cargo build --release
```

Run it locally with the environment variables from above set. The last known IP per record is stored in `/data/last_ip.txt` (one `hostname ip` pair per line).

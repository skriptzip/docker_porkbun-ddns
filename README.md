# <img src="https://porkbun.com/images/porkbun-logo-1200x1200.png" height="32" style="vertical-align:middle; margin-right:8px;"/> Porkbun DDNS

A lightweight Docker-based Dynamic DNS updater for Porkbun domains. This service continuously monitors your public IP address and automatically updates your Porkbun DNS records when changes are detected.

## Features

- **Automatic DNS Updates** - Detects IP changes and updates Porkbun DNS records
- **Docker Ready** - Pre-configured with Docker and Docker Compose
- **Configurable** - Customizable check intervals and IP detection sources
- **Persistent State** - Tracks the last known IP to avoid unnecessary API calls

## Prerequisites

- Docker and Docker Compose
- Porkbun account with API credentials
- A domain registered with Porkbun

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
DOMAIN=example.com
SUBDOMAIN=@
CHECK_INTERVAL=300
```

**Configuration Options:**

- `PORKBUN_API_KEY` - Your Porkbun API key (required)
- `PORKBUN_SECRET_API_KEY` - Your Porkbun API secret key (required)
- `DOMAIN` - Your domain name (required, e.g., `example.com`)
- `SUBDOMAIN` - The subdomain to update (required, use `@` for root domain, or specific subdomain like `www`)
- `CHECK_INTERVAL` - Time in seconds between IP checks (optional, default: `300`)
- `IP_CHECK_URL` - Custom IP detection service (optional, default: `https://api.ipify.org`)

### 3. Run with Docker Compose

```bash
docker-compose up -d
```

This will:
- Build the Docker image
- Start the DDNS service in the background
- Automatically restart on failure
- Mount the `data/` directory for persistent IP tracking

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
2. Restart the service:
   ```bash
   docker-compose restart ddns
   ```

## Environment Variables Explained

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `PORKBUN_API_KEY` | Yes | - | Your Porkbun API key |
| `PORKBUN_SECRET_API_KEY` | Yes | - | Your Porkbun API secret key |
| `DOMAIN` | Yes | - | Your domain name (e.g., `example.com`) |
| `SUBDOMAIN` | Yes | - | Subdomain to update (`@` for root, or specific subdomain) |
| `CHECK_INTERVAL` | No | 300 | Seconds between IP checks |
| `IP_CHECK_URL` | No | https://api.ipify.org | Service to check public IP |

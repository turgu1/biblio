# Docker Configuration Guide

## Overview

Biblio supports both standard Linux deployment and Docker containerized deployment through the `APP_IN_DOCKER` environment variable. This guide explains how configuration works in both modes.

## Configuration Location Strategy

### Standard Mode (Linux/Development)
- **Environment Variable**: `APP_IN_DOCKER` not set or `false`
- **Config File Location**: `config.yaml` in the current working directory
- **Relative Paths Base**: Current working directory

### Docker Mode (Container)
- **Environment Variable**: `APP_IN_DOCKER=true`
- **Config File Location**: `/config/config.yaml` (mounted volume)
- **Relative Paths Base**: `/config` directory

## Quick Start - Standard Mode

```bash
# 1. Copy configuration template
cp config.yaml.example config.yaml

# 2. Edit for your environment
nano config.yaml

# 3. Run the application
cargo run --release
# OR
./target/release/biblio
```

Example `config.yaml`:
```yaml
library_path: "/home/user/calibre-libraries"
service_ip_and_port: "0.0.0.0:8433"
users_file_path: "users.ids"
use_https: false
```

## Quick Start - Docker Mode

### Option 1: Using Docker Compose (Recommended)

```bash
# 1. Create a config directory
mkdir -p ~/biblio-config
cp config.yaml.example ~/biblio-config/config.yaml

# 2. Edit the configuration
nano ~/biblio-config/config.yaml

# Example config for Docker:
# library_path: "/calibre-libraries"  # Matches Docker mount
# users_file_path: "users.ids"        # Relative to /config
# certificate_path: "certs/cert.pem"  # Relative to /config

# 3. Create certs subdirectory if using HTTPS
mkdir -p ~/biblio-config/certs

# 4. Update compose.yaml
nano compose.yaml

# Update the volumes section:
# volumes:
#   - ~/biblio-config:/config:rw
#   - /path/to/calibre:/calibre-libraries:rw

# 5. Start the container
docker compose up -d

# 6. Verify
docker compose logs -f web-server
```

### Option 2: Manual Docker Run

```bash
docker build -t biblio .

docker run -p 8433:8433 \
  -e APP_IN_DOCKER=true \
  -e RUST_LOG=info \
  -v ~/biblio-config:/config:rw \
  -v /path/to/calibre:/calibre-libraries:rw \
  biblio
```

## Configuration File Details

### Path Resolution

Relative paths in `config.yaml` are resolved differently based on the deployment mode:

**Standard Mode**:
```yaml
library_path: "calibre-libraries"  # Resolved as: <working-dir>/calibre-libraries
users_file_path: "users.ids"       # Resolved as: <working-dir>/users.ids
certificate_path: "certs/cert.pem" # Resolved as: <working-dir>/certs/cert.pem
```

**Docker Mode**:
```yaml
library_path: "calibre-libraries"  # Resolved as: /config/calibre-libraries
users_file_path: "users.ids"       # Resolved as: /config/users.ids
certificate_path: "certs/cert.pem" # Resolved as: /config/certs/cert.pem
```

Absolute paths work the same in both modes:
```yaml
library_path: "/calibre-libraries"  # Always /calibre-libraries
certificate_path: "/etc/biblio/cert.pem"  # Always /etc/biblio/cert.pem
```

### Configuration Options

**library_path** (string)
- Where Calibre libraries are stored
- Use absolute path or relative to base directory
- Example (Docker): `/calibre-libraries` or `calibre-libraries` (relative to /config)

**service_ip_and_port** (string)
- Server binding address
- Format: `"IP:PORT"`
- Example: `"0.0.0.0:8433"` (listen all interfaces)

**users_file_path** (string)
- User credentials file
- Format: `username:password_hash:role:email:created_at` (one per line)
- Example (Docker): `users.ids` (relative to /config)

**use_https** (boolean)
- Enable HTTPS/TLS
- Default: `false`

**certificate_path** (string)
- PEM-format X.509 certificate
- Required if `use_https: true`
- Example (Docker): `certs/cert.pem` (relative to /config)

**private_key_path** (string)
- PEM-format private key
- Required if `use_https: true`
- Example (Docker): `certs/key.pem` (relative to /config)

## Docker Volume Mounting

### Typical Docker Compose Setup

```yaml
services:
  web-server:
    build:
      context: .
      target: final
    environment:
      - APP_IN_DOCKER=true
      - RUST_LOG=info
    volumes:
      # Mount config directory (read-write for flexibility)
      - /path/to/biblio-config:/config:rw
      
      # Mount Calibre libraries (read-only recommended)
      - /path/to/calibre-libraries:/calibre-libraries:ro
    ports:
      - "8433:8433"
```

### File Layout Example

Assuming `/home/user/biblio-config` is your config directory:

```
~/biblio-config/
├── config.yaml          # Main configuration file
├── users.ids            # User credentials
└── certs/
    ├── cert.pem         # SSL certificate
    └── key.pem          # SSL private key
```

In `config.yaml`, you'd reference them as:
```yaml
users_file_path: "users.ids"
certificate_path: "certs/cert.pem"
private_key_path: "certs/key.pem"
```

## Switching Between Modes

### Standard → Docker

1. Copy your `config.yaml` to a new directory
2. Move `users.ids` and `certs/` to this directory
3. Update paths in `config.yaml` to be relative:
   - `/home/user/calibre-libraries` → `/calibre-libraries`
   - `/home/user/users.ids` → `users.ids`
4. Set `APP_IN_DOCKER=true` in environment
5. Mount the directory to `/config` in Docker

### Docker → Standard

1. Copy `config.yaml` from the mounted volume
2. Update paths to absolute or working-directory-relative paths
3. Move `users.ids` and `certs/` to working directory
4. Ensure `APP_IN_DOCKER` is not set or is `false`
5. Run from the directory containing `config.yaml`

## Environment Variables

### APP_IN_DOCKER
- **Type**: String
- **Values**: `"true"` or any other value (treated as false)
- **Default**: Unset (treated as false, standard mode)
- **Purpose**: Determines config file location and path resolution base

### RUST_LOG
- **Type**: String
- **Examples**: `info`, `debug`, `error`, `biblio=debug`
- **Default**: Not set
- **Purpose**: Controls logging verbosity

## Troubleshooting

### "Failed to initialize configuration"

**Standard Mode**:
- Ensure `config.yaml` exists in current working directory
- Verify file is valid YAML
- Check file permissions

**Docker Mode**:
- Ensure `/config/config.yaml` exists (check volume mount)
- Verify `APP_IN_DOCKER=true` is set
- Check Docker volume mount with: `docker inspect <container-name>`
- View logs: `docker logs <container-name>`

### Path Not Found Errors

**Standard Mode**:
- Use `pwd` to check working directory
- Use absolute paths if relative paths don't work
- Example: `library_path: "/home/user/calibre"` or `library_path: "calibre"` (relative)

**Docker Mode**:
- Check that volumes are correctly mounted in `compose.yaml`
- Use absolute paths for clarity: `/calibre-libraries`, `/config/users.ids`
- Verify mount with: `docker exec <container> ls -la /config`

### Configuration Not Updating

- Application loads config at startup only
- After changing `config.yaml`, restart the application:
  - Standard: Stop and run again
  - Docker: `docker compose restart web-server`

## Security Considerations

- **users.ids**: Contains password hashes - keep in secure location
- **certs/key.pem**: Keep private key file secure (use `600` permissions)
- **config.yaml**: May contain sensitive paths - consider permissions
- In Docker: Use read-only mounts where possible for Calibre libraries
- In production: Use strong SSL certificates from trusted CAs

## Examples

### Example 1: Standard Linux Setup

```bash
# Create directory structure
mkdir -p ~/.biblio/certs

# Copy config
cp config.yaml.example ~/.biblio/config.yaml

# Edit config for your paths
cat > ~/.biblio/config.yaml << EOF
library_path: "/home/username/calibre-libraries"
service_ip_and_port: "0.0.0.0:8433"
users_file_path: "users.ids"
use_https: false
EOF

# Create users file
echo "admin:argon2id$v=19$m=19456,t=2,p=1$..." > ~/.biblio/users.ids

# Run from config directory
cd ~/.biblio
~/biblio/target/release/biblio
```

### Example 2: Docker with Relative Paths

```bash
# Create config directory
mkdir -p ~/docker-biblio/{certs,libraries}

# Copy config
cp config.yaml.example ~/docker-biblio/config.yaml

# Edit for Docker (all paths relative to /config)
cat > ~/docker-biblio/config.yaml << EOF
library_path: "/calibre-libraries"
service_ip_and_port: "0.0.0.0:8433"
users_file_path: "users.ids"
use_https: true
certificate_path: "certs/cert.pem"
private_key_path: "certs/key.pem"
EOF

# Create users and certs
echo "admin:argon2id$..." > ~/docker-biblio/users.ids

# Generate self-signed cert (optional)
openssl req -x509 -newkey rsa:4096 -keyout ~/docker-biblio/certs/key.pem \
  -out ~/docker-biblio/certs/cert.pem -days 365 -nodes

# Update and run compose
docker compose up -d
```

### Example 3: Docker with Absolute Paths

```yaml
# config.yaml
library_path: "/calibre-libraries"
users_file_path: "/config/users.ids"
certificate_path: "/etc/biblio/cert.pem"
private_key_path: "/etc/biblio/key.pem"
```

This works well if you're mounting different volumes for different purposes.

## Related Documentation

- See [README.md](../README.md) for general setup instructions
- See [AUTHENTICATION.md](./AUTHENTICATION.md) for user management
- See [EXAMPLE_SETUP.md](./EXAMPLE_SETUP.md) for detailed setup walkthrough
- See [config.yaml.example](../config.yaml.example) for configuration reference

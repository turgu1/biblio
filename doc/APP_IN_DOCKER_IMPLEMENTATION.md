# APP_IN_DOCKER Implementation Summary

## Overview

Implemented support for the `APP_IN_DOCKER` environment variable to segregate configuration loading between standard Linux and Docker containerized deployments. This eliminates the need for multiple configuration variations for different deployment scenarios.

## Changes Made

### 1. Core Configuration Module (`src/config.rs`)

**Key Changes**:
- Added `get_base_dir()` method to determine configuration base directory:
  - Docker mode: `/config` when `APP_IN_DOCKER=true`
  - Standard mode: Current working directory otherwise
  
- Added `resolve_path()` method to handle path resolution:
  - Absolute paths used as-is
  - Relative paths resolved relative to base directory
  
- Modified `Config::load()` to:
  - Resolve all paths after YAML deserialization
  - Support both relative and absolute paths
  
- Enhanced `init()` function to:
  - Log deployment mode (Docker or Standard)
  - Provide context-aware error messages

**Code Features**:
- Automatic path resolution based on deployment context
- Single configuration file works for both modes
- Backward compatible with absolute paths

### 2. Application Entry Point (`src/main.rs`)

**Key Changes**:
- Determine config path at startup based on `APP_IN_DOCKER`:
  - Docker: `/config/config.yaml`
  - Standard: `config.yaml`
  
- Enhanced error messages for different deployment modes:
  - Docker: "Ensure config.yaml exists at /config/config.yaml (mounted volume)"
  - Standard: "Ensure config.yaml exists in the working directory"

### 3. Configuration Files

**config.yaml**:
- Added documentation comments explaining location and path resolution
- Clarified relative vs. absolute path behavior
- Updated example values with deployment notes

**config.yaml.example**:
- Comprehensive rewrite with dual-deployment guidance
- Examples for both standard and Docker modes
- Clear path resolution documentation
- Certificate generation methods included
- Deployment location instructions

### 4. Documentation Updates

#### README.md
- Added "Configuration Location" section explaining deployment modes
- Updated "Initial Setup" with Docker-specific guidance
- Enhanced configuration options documentation with examples
- Updated Docker setup instructions with volume mount details
- Added environment variable explanation

#### AUTHENTICATION.md
- Updated "Configuration" section with deployment-aware setup
- Added Docker vs. standard mode examples
- Enhanced troubleshooting with deployment-specific guidance
- Added notes about file location requirements

#### EXAMPLE_SETUP.md
- Added standard mode clarification
- Noted Docker deployment configuration requirements
- Simplified relative path examples

#### New File: DOCKER_CONFIG_GUIDE.md
- Comprehensive Docker configuration guide
- Quick start for both modes
- Volume mounting strategies
- Path resolution explanations
- Troubleshooting section
- Multiple working examples
- Security considerations

## How It Works

### Path Resolution Logic

```
When Application Starts:
  1. Check APP_IN_DOCKER environment variable
  2. If true:
     - Look for config.yaml at /config/config.yaml
     - Resolve relative paths from /config
     - Resolve absolute paths normally
  3. If false or unset:
     - Look for config.yaml in current working directory
     - Resolve relative paths from current working directory
     - Resolve absolute paths normally
```

### Example Scenarios

**Standard Linux**:
```bash
# Working directory: ~/biblio
$ pwd
/home/user/biblio

$ cat config.yaml
library_path: "calibre-libraries"  # Resolves to: ~/biblio/calibre-libraries
users_file_path: "users.ids"       # Resolves to: ~/biblio/users.ids

$ ./target/release/biblio
# INFO: Running in standard mode
```

**Docker Container**:
```bash
# In docker-compose.yaml
volumes:
  - ~/biblio-config:/config:rw

# In /config/config.yaml
library_path: "calibre-libraries"  # Resolves to: /config/calibre-libraries
users_file_path: "users.ids"       # Resolves to: /config/users.ids

# Container startup
$ APP_IN_DOCKER=true ./biblio
# INFO: Running in Docker mode (APP_IN_DOCKER=true)
```

## Verification

### Compilation
- ✅ `cargo check` passes
- ✅ `cargo build --release` completes successfully
- ✅ No warnings or errors

### Runtime Testing
- ✅ Standard mode starts correctly with config.yaml in working directory
- ✅ Logs show "Running in standard mode (APP_IN_DOCKER not set or false)"
- ✅ Docker mode fails gracefully with helpful error message when `/config/config.yaml` doesn't exist
- ✅ Configuration values loaded and accessible to application modules

### Error Handling
- ✅ Clear error messages for missing config files
- ✅ Context-specific guidance in error messages
- ✅ Proper logging of deployment mode

## Benefits

1. **Single Configuration File**: Same `config.yaml` works for both deployments
2. **Flexibility**: Mix of relative and absolute paths supported
3. **Clear Separation**: Docker vs. standard modes clearly distinguished
4. **Backward Compatible**: Existing absolute paths continue to work
5. **Maintenance**: Updates to config handling benefit both modes automatically
6. **Documentation**: Clear guidance for users on both deployment types

## Deployment Checklist

### Standard Linux
- [ ] Copy `config.yaml.example` to `config.yaml`
- [ ] Update `library_path` (absolute or relative to working directory)
- [ ] Update other paths as needed
- [ ] Run from directory containing `config.yaml`
- [ ] Verify "Running in standard mode" in logs

### Docker
- [ ] Create config directory (e.g., `/home/user/biblio-config`)
- [ ] Copy `config.yaml.example` to config directory
- [ ] Update `library_path` to Docker mount paths
- [ ] Update `compose.yaml` volume mounts
- [ ] Create subdirectories (users.ids, certs/) as needed
- [ ] Set `APP_IN_DOCKER=true` in environment (already in compose.yaml)
- [ ] Run `docker compose up -d`
- [ ] Verify "Running in Docker mode" in logs

## Files Modified

1. `src/config.rs` - Core path resolution logic
2. `src/main.rs` - Config file path determination
3. `config.yaml` - Updated documentation
4. `config.yaml.example` - Comprehensive rewrite
5. `README.md` - Updated sections (4 locations)
6. `doc/AUTHENTICATION.md` - Updated sections (2 locations)
7. `doc/EXAMPLE_SETUP.md` - Updated section (1 location)
8. `doc/DOCKER_CONFIG_GUIDE.md` - New comprehensive guide

## Testing Recommendations

1. **Standard Mode**:
   ```bash
   cd ~/biblio
   ./target/release/biblio
   # Verify: "Running in standard mode" appears in logs
   ```

2. **Docker Mode**:
   ```bash
   docker compose up -d
   docker compose logs web-server
   # Verify: "Running in Docker mode" appears in logs
   ```

3. **Path Resolution**:
   - Verify relative paths resolve correctly
   - Test with absolute paths
   - Test with missing files (error messages)

4. **Configuration Updates**:
   - Change `config.yaml` values
   - Restart application
   - Verify new values are used

## Future Enhancements

- Configuration validation schema
- Automatic migration for existing configs
- Configuration reload without restart (requires additional locking)
- Environment variable overrides for sensitive values

# Biblio Authentication Implementation

## Overview

Login capability has been implemented using Argon2 password hashing and credential validation. User credentials are stored in a `users.ids` file.

## Files Modified/Created

### New Files
- **src/auth.rs** - Authentication module
  - User credential management
  - Password hashing with Argon2
  - Password verification
  - User authentication function
  - File-based credential loading

- **users.ids** - User credentials file
  - Format: `username:password_hash` (one per line)
  - Comments start with `#`
  - Secure credential storage

### Modified Files
- **Cargo.toml** - Added dependencies:
  - `actix-session = "0.9"` - Session management
  - `actix-identity = "0.7"` - Identity handling
  - `argon2 = "0.5"` - Password hashing
  - `rand = "0.8"` - Random number generation for salts

- **src/config.rs** - Added configuration:
  - `USERS_FILE_PATH: &str` - Path to users.ids file (default: "./users.ids")

- **src/main.rs** - Updated to:
  - Load users from users.ids file
  - Pass users to API handlers
  - Log authentication initialization status

- **src/api.rs** - Added authentication endpoints:
  - `POST /api/auth/login` - Login with username/password
  - `POST /api/auth/logout` - Logout endpoint
  - `GET /api/auth/current-user` - Get current user info

## API Endpoints

### Login
**POST /api/auth/login**

Request:
```json
{
  "username": "admin",
  "password": "admin"
}
```

Success Response (200):
```json
{
  "success": true,
  "data": {
    "username": "admin"
  },
  "error": null
}
```

Error Response (401):
```json
{
  "success": false,
  "data": null,
  "error": "Invalid credentials"
}
```

### Logout
**POST /api/auth/logout**

Response (200):
```json
{
  "success": true,
  "data": {
    "message": "logged out"
  },
  "error": null
}
```

### Get Current User
**GET /api/auth/current-user**

Response (401 - not authenticated):
```json
{
  "success": false,
  "data": null,
  "error": "Not authenticated"
}
```

## Users File Format

### users.ids

```
# Biblio users file
# Format: username:password_hash
# 
# Generate hashes using: auth::hash_password("password")
# Or use any Argon2 password hashing tool

# Default admin user (password: admin)
admin:$argon2id$v=19$m=19456,t=2,p=1$R9U3cWcyK8GJJYtW4qfK9Q$TrqkMfKcXcCE0W0xY+eT7F3Y0Q0xY+eT7F3Y0Q0xY+0

# Add more users in the same format
# user1:$argon2id$...
# user2:$argon2id$...
```

## How to Add New Users

### Option 1: Using Rust Code
Create a small utility to hash passwords:

```rust
use biblio::auth;

fn main() {
    match auth::hash_password("mypassword") {
        Ok(hash) => println!("newuser:{}", hash),
        Err(e) => println!("Error: {}", e),
    }
}
```

Then add the output to users.ids file.

### Option 2: Using CLI Tools
Install and use Argon2 CLI:
```bash
argon2 "mypassword" -t 2 -m 19456 -p 1 -l 32
```

Then format as: `username:$argon2id$...` in users.ids

## Configuration

### Change Users File Location

Edit `src/config.rs`:
```rust
pub const USERS_FILE_PATH: &str = "/path/to/users.ids";
```

Then rebuild:
```bash
cargo build --release
```

## Logging

The authentication module logs the following events:

**Debug Level**
- User loaded successfully
- User authenticated successfully
- Successful book count retrieval

**Warning Level**
- No metadata.db found in directory
- Failed library metadata creation
- Failed authentication attempt for a user
- Authentication attempt for non-existent user

**Error Level**
- Failed to open database at path
- Failed to access tables in database
- Failed to open database for library
- Users file not found
- No valid users found in users file
- Failed to load users

Enable debug logging:
```bash
RUST_LOG=debug cargo run --release
```

## Security Considerations

1. **Password Hashing**: Uses Argon2id v19 with:
   - Memory: 19456 KiB
   - Time: 2 iterations
   - Parallelism: 1

2. **Salt Generation**: Random salts generated for each password using `rand::rngs::OsRng`

3. **File Security**:
   - Keep users.ids in a secure location
   - Do not commit to version control
   - Restrict file permissions to readable by application only
   - Consider encrypting at rest in production

4. **Session Security**:
   - Sessions are currently stateless (no session storage)
   - Clients must authenticate with each request or implement frontend session management
   - Production deployment should use secure HTTPS only

## User Management

Admin users can manage other users, passwords, and roles through the Admin Dashboard.

### Admin Dashboard (`/admin.html`)

The Admin Dashboard provides a web interface for managing users with the following features:

- **List Users**: View all registered users with their roles, emails, and creation dates
- **Create User**: Add new users with configurable roles
- **Reset Password**: Change any user's password without knowing their current password
- **Delete User**: Remove users from the system (except the admin user)
- **Edit User Role**: Update user roles (admin, librarian, user, reader)

### Admin API Endpoints

#### List All Users
**GET /api/admin/users**

Response (200):
```json
{
  "success": true,
  "data": [
    {
      "username": "john",
      "role": "librarian",
      "email": "john@example.com",
      "created_at": "2024-01-15T10:30:00Z"
    },
    {
      "username": "jane",
      "role": "user",
      "email": "jane@example.com",
      "created_at": "2024-01-16T14:22:00Z"
    }
  ],
  "error": null
}
```

#### Create New User
**POST /api/admin/users**

Request:
```json
{
  "username": "newuser",
  "password": "SecurePass123!",
  "role": "user",
  "email": "newuser@example.com"
}
```

Response (201):
```json
{
  "success": true,
  "data": {
    "username": "newuser",
    "role": "user",
    "email": "newuser@example.com",
    "created_at": "2024-01-17T09:15:00Z"
  },
  "error": null
}
```

#### Update User
**PUT /api/admin/users/{username}**

Request:
```json
{
  "role": "librarian",
  "email": "updated@example.com"
}
```

Response (200):
```json
{
  "success": true,
  "data": {
    "username": "newuser",
    "role": "librarian",
    "email": "updated@example.com",
    "created_at": "2024-01-17T09:15:00Z"
  },
  "error": null
}
```

#### Reset User Password (Admin Only)
**POST /api/admin/users/{username}/password**

Request:
```json
{
  "username": "newuser",
  "new_password": "NewSecurePass456!"
}
```

Response (200):
```json
{
  "success": true,
  "data": {
    "message": "Password reset successfully"
  },
  "error": null
}
```

#### Delete User
**DELETE /api/admin/users/{username}**

Response (200):
```json
{
  "success": true,
  "data": {
    "message": "User newuser deleted"
  },
  "error": null
}
```

Note: The admin user cannot be deleted.

## User Self-Service

Users can manage their own profiles and passwords through the User Profile page.

### User Profile Page (`/profile.html`)

The User Profile page provides:

- **User Information**: Display current username, role, and email
- **Change Password**: Self-service password change with strength validation
- **Password Requirements**: Real-time validation showing:
  - Minimum 8 characters
  - At least one uppercase letter
  - At least one lowercase letter
  - At least one digit
  - At least one special character
- **Password Confirmation**: Ensures password matches before submission

### User API Endpoint

#### Change Own Password
**POST /api/auth/change-password**

Request:
```json
{
  "username": "john",
  "current_password": "OldPass123!",
  "new_password": "NewPass456!"
}
```

Response (200):
```json
{
  "success": true,
  "data": {
    "message": "Password changed successfully"
  },
  "error": null
}
```

Error Response (401):
```json
{
  "success": false,
  "data": null,
  "error": "Invalid current password"
}
```

## User Roles

The system supports four role levels with different permissions:

| Role | Permissions | Use Case |
|------|-------------|----------|
| **Admin** | Manage users, reset passwords, change roles, view audit logs | System administrators |
| **Librarian** | Manage libraries, refresh metadata | Library curators |
| **User** | Browse all libraries, download books | Regular users |
| **Reader** | Browse and view books (default) | Guest users |

### Role-Based Permissions

```rust
// Admin Permissions
- can_manage_users() → true
- can_manage_libraries() → true
- can_view_audit_logs() → true
- can_manage_permissions() → true

// Librarian Permissions
- can_manage_libraries() → true
- can_browse_libraries() → true

// User Permissions
- can_browse_libraries() → true

// Reader Permissions
- can_browse_libraries() → true
```

## Future Enhancements

### Completed ✅

1. **Authorization** - Role-based access control:
   - ✅ User roles (admin, librarian, user, reader)
   - ✅ Per-library access control via RBAC module
   - ✅ Endpoint authorization via permission checking

2. **Password Management**:
   - ✅ Password change endpoint (user self-service)
   - ✅ Password reset functionality (admin-only)
   - ✅ Password strength validation with requirements

3. **User Management**:
   - ✅ Add/remove users via API endpoints
   - ✅ User profile endpoints and web UI
   - ✅ User roles and metadata storage

4. **Audit Logging**:
   - ✅ Track login/logout attempts
   - ✅ Log password changes and resets
   - ✅ Log user management operations
   - ✅ Monitor access patterns with audit log retrieval

### Future Enhancements

1. **Session Storage** - Implement persistent sessions with:
   - Database-backed session store
   - Session timeout/expiration
   - Cookie-based session tokens

2. **User Preferences**:
   - User preference storage (theme, language, etc.)
   - Per-user library access restrictions
   - Reading history and bookmarks

3. **Multi-factor Authentication**:
   - TOTP/authenticator app support
   - Hardware key support
   - Email-based 2FA

4. **User Import/Export**:
   - Batch user import from CSV
   - User data export functionality
   - User activity reports

## Testing

### Test Login
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'
```

### Test Logout
```bash
curl -X POST http://localhost:8080/api/auth/logout
```

### Test Current User
```bash
curl -X GET http://localhost:8080/api/auth/current-user
```

### Test Change Password
```bash
curl -X POST http://localhost:8080/api/auth/change-password \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","current_password":"admin","new_password":"NewPass123!"}'
```

### Test Admin: List Users
```bash
curl -X GET http://localhost:8080/api/admin/users
```

### Test Admin: Create User
```bash
curl -X POST http://localhost:8080/api/admin/users \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","password":"NewPass123!","role":"user","email":"newuser@example.com"}'
```

### Test Admin: Reset Password
```bash
curl -X POST http://localhost:8080/api/admin/users/newuser/password \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","new_password":"ResetPass456!"}'
```

### Test Admin: Delete User
```bash
curl -X DELETE http://localhost:8080/api/admin/users/newuser
```

## Troubleshooting

**"Users file not found"**
- Verify USERS_FILE_PATH in config.rs is correct
- Ensure users.ids exists in the configured location
- Check file permissions

**"No valid users found"**
- Verify users.ids has at least one valid entry
- Check format: `username:password_hash:role:email:created_at` (role and metadata optional)
- Ensure lines don't have spaces around colons
- Remove or fix malformed entries

**"Authentication error"**
- Verify password hash format is valid Argon2
- Check username and password in request
- Verify users.ids file is readable

**"Failed to load users"**
- Check logs with `RUST_LOG=error` for detailed error messages
- Verify file path and permissions
- Ensure file content is valid UTF-8

**Admin Dashboard Access Denied**
- Ensure your user account has `admin` role
- Check that the role is correctly set in users.ids file
- Refresh the page or clear browser cache

## File Statistics

- **src/auth.rs**: ~250 lines - Authentication module with persistence functions
- **src/api.rs**: ~320 lines - API endpoints including user management
- **src/audit.rs**: ~130 lines - Audit logging with extended event types
- **src/rbac.rs**: ~100 lines - Role-based access control
- **src/session.rs**: ~90 lines - Session management
- **src/config.rs**: ~30 lines - Configuration
- **public/index.html**: ~560 lines - Main library browser
- **public/admin.html**: ~500 lines - Admin user management dashboard
- **public/profile.html**: ~450 lines - User profile and password management
- **public/app.js**: ~1340 lines - Application logic
- **Cargo.toml**: Added dependencies (actix-session, actix-identity, argon2, rand, chrono, uuid, serde, tracing)
- **users.ids**: Template with default admin user

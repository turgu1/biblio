# Biblio - Admin Features Implementation

## Overview

Role-based access control (RBAC) and admin user management system have been implemented for Biblio, enabling administrators to manage users, roles, and passwords through a dedicated admin panel.

---

## ✅ Completed Features

### 1. User Authentication & Role Management

#### Role System
Four user roles are supported:
- **Admin** - Full access to user management, password reset, user listing
- **Librarian** - Library management capabilities (future expansion)
- **User** - Standard user with reading capabilities
- **Reader** - Read-only access to libraries

#### User Storage
- File-based storage at `users.ids`
- Format: `username:password_hash:role:email:created_at`
- Password hashing: Argon2id v19 with parameters (m=19456, t=2, p=1)
- ISO 8601 timestamps with timezone information
- File parsing uses `splitn(5, ':')` to handle colons in timestamps

### 2. Backend Admin API Endpoints

All admin endpoints validate the requesting user has admin role and return `403 Forbidden` if unauthorized.

#### User Management
- **POST** `/api/admin/users` - Create new user
  - Request: `{username, password, role, email, admin_username}`
  - Response: Success message with created user details
  - Validation: Check username doesn't exist, admin_username must be admin

- **GET** `/api/admin/users` - List all users
  - Query param: `admin_username` (requesting admin username)
  - Response: Array of user objects {username, role, email, created_at}
  - Returns 403 if requesting user not admin

- **PUT** `/api/admin/users/{username}` - Update user
  - Request: `{role, email, admin_username}`
  - Response: Success message with updated user details
  - Validation: Cannot update if requesting user not admin

- **DELETE** `/api/admin/users/{username}` - Delete user
  - Request: `{admin_username}`
  - Response: Success message
  - Validation: Cannot delete own account, must be admin

#### Password Management
- **POST** `/api/admin/users/{username}/password` - Reset user password
  - Request: `{username, new_password, admin_username}`
  - Response: Success message
  - Validation: admin_username must have admin role
  - Note: One admin can reset another admin's password

#### Authorization
- `verify_admin_user()` helper (line ~785 in src/api.rs)
  - Loads users from file
  - Checks user exists and has role=="admin"
  - Called by all 5 admin endpoints
  - Returns 403 Forbidden if not authorized

### 3. Frontend Admin Panel

#### Admin Button Visibility
- Located in top command bar of main interface [index.html](../public/index.html#L393)
- Button HTML: `<button id="adminBtn" style="display: none;">🔐 Admin</button>`
- **Hidden by default for all users**
- **Only visible for users with admin role**
- Implementation in [app.js](../public/app.js#L165):
  - `checkAdminStatus()` reads role from localStorage
  - Sets `display: inline-block` for admin role only
  - Sets `display: none` for all other roles
  - Called on app initialization and after login
  - Also hidden explicitly on logout

#### Admin Panel (admin.html)

##### Page Protection
- `protectAdminPage()` function redirects non-admin users to home page
- Checks role from localStorage key `'biblio_auth'`
- Called on page load in DOMContentLoaded event
- Returns 403 error if user attempts direct URL access without admin role

##### Add User Form
- Form inputs: Username, Password, Role (dropdown), Email
- Validates:
  - Username not empty and unique
  - Password strength requirements
  - Role selection
  - Email format (if provided)
- Sends request with admin_username in body
- Shows success/error notifications
- Clears form on successful creation

##### Users List Table
- Displays all users with columns:
  - Username
  - Role
  - Email (or "-" if not provided)
  - Created date
  - Actions (left-aligned per UX requirements)
- Actions per user:
  - **Edit** - Change role and email
  - **Reset Password** - Reset user password (admin only)
  - **Delete** - Remove user (cannot delete own account)
- Table styling:
  - Responsive with hover effects
  - Actions column header: left-justified via CSS `text-align: left`
  - Other columns: center-aligned

##### Password Reset Form
- Modal dialog for resetting user password
- Fields: Username, New Password, Confirm Password
- Validation: Passwords must match
- Sends request with admin_username and new_password
- Shows confirmation message on success

##### Edit User Form
- Modal dialog for updating user role and email
- Pre-populated with current user data
- Allows changing:
  - Role (dropdown select)
  - Email address
- Prevents self-deletion (checks if deleting own account)
- Shows confirmation on success

### 4. Frontend Authentication State Management

#### localStorage Usage
- Key: `'biblio_auth'`
- Structure: `{isAuthenticated, username, role, timestamp}`
- Stored during login with user's actual role from backend
- Used for:
  - Admin button visibility control
  - Profile page role display
  - Admin page protection
  - All admin API calls

#### Login Endpoint
- Updated to return user's role in response
- Response format: `{success: true, data: {username, role}, error: null}`
- Frontend extracts role and stores in localStorage

#### Session Persistence
- Role persists across page reloads via localStorage
- Cleared on logout
- Timestamp tracks session creation
- No role expiration (valid until logout or manual session clear)

### 5. Audit Logging

#### Unauthorized Access Attempts
- All failed admin authorization attempts logged as `UnauthorizedAccess` events
- Includes:
  - Timestamp of attempt
  - Requested admin action
  - Attempted by username
  - Request details

#### Logged Events
- Create user without admin role
- Update user without admin role
- Delete user without admin role
- Reset password without admin role
- List users without admin role

---

## 🔧 Implementation Details

### Backend Changes (src/api.rs)

#### Request Structures
```rust
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub email: Option<String>,
    pub admin_username: Option<String>,
}

pub struct UpdateUserRequest {
    pub role: Option<String>,
    pub email: Option<String>,
    pub admin_username: Option<String>,
}

pub struct AdminChangePasswordRequest {
    pub username: String,
    pub new_password: String,
    pub admin_username: Option<String>,
}

pub struct DeleteUserRequest {
    pub admin_username: Option<String>,
}
```

#### Helper Function
```rust
async fn verify_admin_user(admin_username: Option<&str>) -> Result<(), String> {
    // Loads users from file
    // Finds user by admin_username
    // Checks if role == "admin"
    // Returns Ok(()) or Err(message)
}
```

#### Critical Bug Fix: User Resurrection Prevention
- **Issue**: Deleted users would reappear after creating new users
- **Root Cause**: `create_user()` was cloning stale in-memory cache from server startup
- **Solution**: Changed `create_user()` to load from file on every call:
  ```rust
  let mut users = auth::load_users(config::USERS_FILE_PATH)?;  // Always load fresh
  // ... modifications ...
  auth::save_users(&users, config::USERS_FILE_PATH)?;  // Save to file
  ```
- **Impact**: Ensures current file data is used for all modifications

### Frontend Changes

#### app.js Updates
- `checkAdminStatus()` - Check role from localStorage and control button visibility
- `saveAuthState()` - Store role during login
- `handleLogin()` - Extract role from login response
- `handleLogout()` - Explicitly hide admin button on logout

#### admin.html Updates
- `protectAdminPage()` - Redirect non-admin users
- `handleAddUser()` - Create new user via admin API
- `loadUsers()` - Fetch and display user list
- `submitResetPassword()` - Reset user password
- `deleteUser()` - Delete user with confirmation
- CSS for Actions column: left-aligned via `:last-child` selector

---

## 🔐 Security Considerations

### Password Hashing
- Algorithm: Argon2id v19 (resistant to GPU attacks)
- Parameters:
  - Memory (m): 19456 KB
  - Time (t): 2 iterations
  - Parallelism (p): 1
- Uses cryptographically secure random salts

### Authorization
- Server-side validation on all admin endpoints
- Cannot escalate privileges (role check mandatory)
- Admin operations require explicit admin_username parameter
- Failed authorization attempts logged as audit events

### File Security
- users.ids contains password hashes (not plaintext)
- File should be readable only by application
- Timestamps prevent tampering with creation dates

### Session Security
- Role stored in localStorage (client-side)
- No bearer tokens or session IDs needed for this implementation
- Frontend validates before sending admin requests
- Backend re-validates on every admin endpoint

---

## 📋 Default Credentials

### Initial Admin User
- **Username**: `admin`
- **Password**: `Admin@Pass123!`
- **Role**: `admin`
- **Created**: 2026-01-16

### Test Users (For Reference)
- **newusertest**: Password `Test@123!@#` (role: reader)
- **verification**: Created 2026-01-17 (role: reader)

---

## ✨ UI/UX Features

### Admin Button
- Located in top navigation bar
- 🔐 Lock icon for visual recognition
- Hidden by default
- Visible only for admin users
- Clicking redirects to admin.html

### Admin Panel Layout
- Responsive design works on desktop and tablet
- Two-column form layout for better usability
- Modal dialogs for sensitive operations (password reset, edit)
- Confirmation dialogs before destructive actions (delete)
- Real-time validation feedback
- Success/error toast notifications

### Users Table
- Striped rows for readability
- Hover effects highlight current row
- Responsive scrolling on small screens
- Actions buttons clearly labeled
- Left-aligned Actions column header per UX best practices

---

## 🧪 Testing Checklist

### Authentication
- [x] Admin can login and access admin panel
- [x] Non-admin users cannot access admin panel
- [x] Admin button hidden for non-admin users
- [x] Admin button visible for admin users
- [x] User role returned correctly on login
- [x] Role persists in localStorage

### Admin Operations
- [x] Create new user with various roles
- [x] List all users with correct details
- [x] Update user role and email
- [x] Reset user password successfully
- [x] Delete user from system
- [x] Cannot delete own admin account

### Authorization
- [x] Non-admin API calls return 403 Forbidden
- [x] Admin operations require admin_username parameter
- [x] Server validates admin role on backend
- [x] Unauthorized attempts logged in audit logs

### UI/UX
- [x] Admin button hidden by default
- [x] Admin button shows only for admins
- [x] Admin button hides on logout
- [x] Admin panel redirects non-admins
- [x] Users list loads correctly
- [x] Actions column left-aligned
- [x] Forms show validation errors
- [x] Success messages appear after operations

---

## 📝 Known Limitations

1. **No Role Hierarchy** - All admins have equal permissions
2. **No Audit Trail UI** - Audit logs exist but no frontend viewer
3. **No Password Policies** - Any password accepted (for testing)
4. **No Email Verification** - Email stored but not verified
5. **No Account Lockout** - No protection against brute force (add in production)
6. **No 2FA** - Single password authentication only
7. **Single Admin Escalation** - Cannot remove all admins from system
8. **File-Based Storage** - No database backend, file-based users.ids

---

## 🚀 Future Enhancements

1. **User Groups** - Group users for bulk permission management
2. **Fine-Grained Permissions** - Per-library access control
3. **Activity Logging** - Full audit trail with UI viewer
4. **Password Policies** - Enforce complexity, expiration, history
5. **Email Notifications** - Send reset links instead of direct reset
6. **Session Management** - Track active sessions, force logout
7. **Rate Limiting** - Prevent brute force attacks
8. **Database Backend** - Replace file storage with proper database

---

## 📖 Related Documentation

- [AUTHENTICATION.md](AUTHENTICATION.md) - Authentication system details
- [FRONTEND_AUTHENTICATION.md](FRONTEND_AUTHENTICATION.md) - Frontend auth implementation
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Overall project implementation
- [README.md](../README.md) - Main project README

---

**Last Updated**: January 16, 2026  
**Status**: Complete  
**Version**: 1.0

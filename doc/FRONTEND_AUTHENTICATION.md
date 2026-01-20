# Frontend Authentication Integration Guide

## Overview

The Biblio web platform now includes a complete authentication system that requires users to login before accessing the e-book library. The frontend handles user authentication through a modern login interface that communicates with the backend API.

## Architecture

### Frontend Components

#### 1. Login Page
- **File**: `public/index.html` & `public/app.js`
- **Design**: Beautiful gradient background with centered login card
- **Elements**:
  - Username input field
  - Password input field
  - Submit button with gradient styling
  - Error message display area

#### 2. Authentication State Management
Located in `public/app.js` BiblioApp class:

```javascript
// Authentication properties
isAuthenticated: boolean      // Current auth status
currentUsername: string       // Logged-in username

// Authentication methods
saveAuthState(username)       // Store auth in localStorage
loadAuthState()              // Retrieve auth from localStorage
clearAuthState()             // Remove auth from localStorage
```

#### 3. Login Flow Handler
**Method**: `handleLogin(e)`
- Validates username and password fields
- Makes POST request to `/api/auth/login`
- Handles success and error responses
- Stores authentication state on success
- Displays error messages on failure
- Transitions to main app view

#### 4. Main App Control
**Method**: `showMainApp()`
- Displays content area and library browser
- Adds user info section to top panel
- Shows logout button with red styling
- Displays current username

#### 5. Logout Handler
**Method**: `handleLogout()`
- Calls `/api/auth/logout` endpoint
- Clears local authentication state
- Removes user info from UI
- Returns to login page

## Data Flow

### Initial Load
```
Page Load
  ↓
JavaScript loads
  ↓
DOMContentLoaded event
  ↓
BiblioApp.init()
  ↓
Check localStorage for auth state
  ├─ If authenticated → loadLibraries() + showMainApp()
  └─ If not authenticated → showLoginPage()
```

### Login Process
```
User submits login form
  ↓
handleLogin() validates inputs
  ↓
POST /api/auth/login
  {
    "username": "admin",
    "password": "admin"
  }
  ↓
Server validates credentials
  ↓
Response received
  ├─ Success (200)
  │   ├─ saveAuthState(username)
  │   ├─ loadLibraries()
  │   ├─ showMainApp()
  │   └─ Display user interface
  │
  └─ Failure (401)
      └─ Display error message
```

### Logout Process
```
User clicks logout button
  ↓
handleLogout() executes
  ↓
POST /api/auth/logout
  ↓
clearAuthState()
  ↓
showLoginPage()
  ↓
Return to login screen
```

## LocalStorage Schema

### Auth State Object
```javascript
{
  "isAuthenticated": true,
  "username": "admin",
  "timestamp": 1642334567890
}
```

**Storage Key**: `biblio_auth`

**Persistence**: Survives browser restart and tab closure

## Login Page UI

### Visual Design
- **Background**: Purple-to-pink gradient (667eea → 764ba2)
- **Card**: White background with shadow
- **Layout**: Centered, responsive max-width 400px

### Form Elements

**Username Input**
- Placeholder: "Enter username"
- No character restrictions
- Sent as-is to backend

**Password Input**
- Type: password (hidden)
- Placeholder: "Enter password"
- Sent as-is to backend

**Submit Button**
- Text: "Login"
- Gradient background matching page
- Triggers form submission

**Error Display**
- Red background (#fadbd8)
- Red text (#c0392b)
- Shows API response errors
- Hidden when no error

## User Info Display

After successful login, the top panel shows:

```
admin [Logout]
```

**Components**:
- Username display
- Logout button (red background)
- Positioned in top-right of navigation bar

## Session Management

### Client-Side
- Authentication state stored in localStorage
- Persists across browser sessions
- Cleared on logout

### Server-Side
- Stateless endpoints (no session storage)
- Each request is independent
- Backend validates credentials on each auth request

### Future Enhancement
- Server-side session storage
- Session tokens/JWT for API requests
- Automatic session expiration
- "Remember Me" functionality

## Error Handling

### Network Errors
```javascript
catch (error) {
    errorDiv.textContent = 'An error occurred during login';
    errorDiv.style.display = 'block';
}
```

### Invalid Credentials
- Server returns: `{"success": false, "error": "Invalid credentials"}`
- Frontend displays error to user
- Does not persist auth state

### Missing Fields
- Frontend validation prevents empty submissions
- Checks username and password before API call

## Security Considerations

1. **Password Transmission**
   - HTTPS recommended in production
   - Passwords sent in request body (not URL)
   - Handled by backend Argon2 hashing

2. **Authentication State**
   - Stored in localStorage (visible to JavaScript)
   - In production, consider HTTP-only cookies
   - Current approach suits single-user or trusted environments

3. **Backend Validation**
   - All credentials validated server-side
   - Argon2id password hashing with random salts
   - Attack logging for failed attempts

4. **Logout**
   - Clears all client-side state
   - Server logout endpoint called
   - User returned to login page

## API Integration

### Login Endpoint
```
POST /api/auth/login
Content-Type: application/json

Request:
{
  "username": "admin",
  "password": "admin"
}

Response (Success):
{
  "success": true,
  "data": {
    "username": "admin"
  },
  "error": null
}

Response (Failure):
{
  "success": false,
  "data": null,
  "error": "Invalid credentials"
}
```

### Logout Endpoint
```
POST /api/auth/logout

Response:
{
  "success": true,
  "data": {
    "message": "logged out"
  },
  "error": null
}
```

## Files Modified

### public/app.js (+255 lines)
- Added authentication properties
- Added auth state management methods
- Added login page UI generation
- Added login form handler
- Added logout handler
- Modified init() method for auth check

### Total Frontend Changes
- **public/app.js**: 1066 → 1321 lines (+255 lines)
- **public/index.html**: Unchanged (CSS/HTML supports auth UI)

## Testing

### Manual Tests Performed
- ✅ Login with valid credentials (admin/admin)
- ✅ Login with wrong password
- ✅ Login with non-existent user
- ✅ Error messages display correctly
- ✅ Successful login shows main app
- ✅ User info displays in top panel
- ✅ Logout button appears and functions
- ✅ Logout clears auth state
- ✅ Returning to page shows login again
- ✅ Auth state persists on refresh

## Usage Examples

### Logging In
1. Open http://localhost:8433
2. See gradient login page
3. Enter username: `admin`
4. Enter password: `admin`
5. Click "Login"
6. Main library interface appears

### Logging Out
1. Click "Logout" button in top panel
2. Auth state cleared
3. Return to login page

### Adding New Users
See `doc/AUTHENTICATION.md` for instructions on:
- Generating password hashes
- Adding users to `users.ids` file
- Creating user credentials

## Troubleshooting

### Login Page Not Appearing
- Check browser console for JavaScript errors
- Verify localStorage is enabled
- Clear browser cache and refresh

### Login Always Fails
- Verify credentials in users.ids file
- Check server logs for error messages
- Ensure backend is running on port 8433

### Session Lost After Refresh
- localStorage may be cleared
- Check browser privacy settings
- Verify localStorage is not disabled

## Future Enhancements

1. **Session Tokens**
   - Implement JWT for API requests
   - Add token refresh mechanism
   - Expire tokens after inactivity

2. **User Management**
   - Create user management interface
   - Allow password changes
   - Support user deletion

3. **Security**
   - Implement rate limiting
   - Add account lockout after failed attempts
   - Support two-factor authentication
   - Add HTTPS enforcement

4. **User Experience**
   - Remember Me functionality
   - Session timeout warnings
   - Auto-logout after inactivity
   - Password reset via email

5. **Advanced Features**
   - Role-based access control (RBAC)
   - Multi-user support with preferences
   - User activity logging
   - Admin user management panel

## References

- Backend authentication: `doc/AUTHENTICATION.md`
- Argon2 hashing: `src/auth.rs`
- API routes: `src/api.rs`
- Frontend app: `public/app.js`

# Biblio - Web-Based E-book Library Browser

A modern web application for browsing multiple Calibre e-book libraries with a responsive panel-based UI.

<img src="pictures/main.png" alt="main" width="800"/>

## Features

- **Multiple Library Support**: Browse multiple Calibre libraries from a single interface
- **Rich Metadata Display**: View detailed information about each book including authors, series, tags, and more
- **Advanced Filtering**: Filter books by authors, tags, and series
- **Search Functionality**: Full-text search across book titles and authors with quick clear button
- **Cover Image Gallery**: Browse book covers in a grid layout with dynamic sizing
  - **Continuous Size Control**: Range slider to adjust cover size from 50px to 250px
  - **Responsive Spacing**: Grid gaps and title positioning automatically adjust with cover size
  - **Dynamic Title Display**: Title font size and line count scale based on available space
  - **Auto-Fill**: Screen automatically fills with available books when resizing or switching libraries
- **Table View**: Tabular display of books with resizable columns and visibility controls
  - **Resizable Columns**: Drag column edges to adjust width
  - **Column Visibility**: Toggle which columns to display
  - **Consistent Formatting**: Author names formatted consistently with filter panel
- **Responsive Design**: Works on desktop, tablet, and mobile devices
- **Infinite Scroll**: Efficiently browse large libraries with progressive loading
- **Session Persistence**: Automatically saves your filters, search, library selection, and view preferences using browser cookies
- **User Authentication**: Secure login system with role-based access control
- **Admin Panel**: Comprehensive user management interface for administrators
- **Role-Based Access Control**: Four-level permission system (Admin, Librarian, User, Reader)
- **User Management**: Create, update, delete users and manage passwords from admin panel

## Architecture

### Backend (Rust)
- **Framework**: Actix-web - High-performance async web framework
- **Database**: SQLite (Calibre native format) - Direct read access to Calibre databases
- **API**: RESTful API endpoints for library and book operations

### Frontend (Web)
- **HTML5**: Modern semantic markup
- **CSS3**: Responsive grid layouts with flexbox
- **JavaScript**: Vanilla ES6+ for dynamic interactions

## Project Structure

```
biblio/
├── Cargo.toml                 # Rust dependencies and project config
├── src/
│   ├── main.rs              # Application entry point and server setup
│   ├── db.rs                # Calibre database access layer
│   ├── library.rs           # Library discovery and scanning
│   └── api.rs               # REST API endpoint handlers
├── public/
│   ├── index.html           # Main web interface with embedded CSS
│   ├── app.js               # Frontend JavaScript application
│   └── favicon.ico          # App icon (optional)
└── libraries/               # Directory for Calibre libraries (auto-created)
    └── YourLibrary/
        └── metadata.db      # Calibre metadata database
```

## Installation

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Calibre libraries with `metadata.db` files

### Option 1: Local Installation

1. **Clone or extract the project**:
   ```bash
   cd <biblio-parent-folder>/biblio
   ```

2. **Configure the library path**:
   - Copy the configuration example file:
     ```bash
     cp src/config.rs.example src/config.rs
     ```
   - Edit `src/config.rs` and set the `LIBRARY_PATH` to your Calibre libraries directory:
     ```rust
     pub const LIBRARY_PATH: &str = "/path/to/your/calibre-libraries";
     ```
   - **Note**: The `src/config.rs` file is local configuration and should not be committed to version control

3. **Build the application**:
   ```bash
   cargo build --release
   ```

4. **Run the server**:
   ```bash
   cargo run --release
   ```
   
   Or use the pre-built binary:
   ```bash
   ./target/release/biblio
   ```

5. **Access the application**:
   - Open your web browser and go to: `http://localhost:8080`

### Option 2: Docker Installation

#### Prerequisites for Docker
- Docker ([Install Docker](https://docs.docker.com/get-docker/))
- Docker Compose ([Install Docker Compose](https://docs.docker.com/compose/install/))

#### Setup Steps

1. **Configure the library path**:
   - Copy the configuration example file:
     ```bash
     cp src/config.rs.example src/config.rs
     ```
   - Edit `src/config.rs` and set the `LIBRARY_PATH` to your Calibre libraries directory:
     ```rust
     pub const LIBRARY_PATH: &str = "/calibre-libraries";
     ```

2. **Update compose.yml** (if needed):
   - Edit `compose.yml` and update the volume mount path for your Calibre libraries:
     ```yaml
     volumes:
       - /your/actual/calibre/path:/calibre-libraries:rw
     ```

3. **Build and run with Docker Compose**:
   ```bash
   docker compose up -d
   ```

4. **Access the application**:
   - Open your web browser and go to: `http://localhost:8080`

#### Docker Commands

- **View logs**:
  ```bash
  docker compose logs -f biblio
  ```

- **Stop the application**:
  ```bash
  docker compose down
  ```

- **Rebuild the image**:
  ```bash
  docker compose build --no-cache
  ```

- **Run with manual Docker**:
  ```bash
  docker build -t biblio .
  docker run -p 8080:8080 \
    -v /your/calibre/path:/calibre-libraries:rw \
    biblio
  ```

## Usage

### Main Interface

The application is organized into five panels:

#### Top Panel (Command Bar)
- **Refresh**: Reload the library list and rescan for new libraries
- **Settings**: Configure application preferences (to be implemented)
- **About**: View version and application information

#### Left Panel (Filters)
- **Libraries**: Switch between available libraries
- **Authors**: Filter books by specific authors
- **Tags**: Filter books by subject tags
- **Series**: Filter books by series

#### Center Panel (Books Grid & Table)
- **Search Box**: Search for books by title or author
  - **Quick Clear**: Click the × button to instantly clear search results
- **View Mode Toggle**: Switch between Grid and Table display modes
- **Cover Size Control** (Grid mode only):
  - **Range Slider**: Adjust cover size from 50px to 250px
  - **Current Size Display**: Shows selected pixel size
  - **Dynamic Scaling**: Spacing and title formatting adjust automatically
- **Column Visibility** (Table mode only):
  - **Visibility Toggles**: Show/hide columns (Title, Authors, Series, Publisher, Rating, Published)
  - **Resizable Columns**: Drag column separators to adjust width
  - **Smart Defaults**: Columns have automatic minimum widths for readability
- **Sort Options**: Sort by recent, title, or author
- **Grid Display**: Visual gallery of book covers with titles
  - **Auto-Fill**: Loads more books to fill available screen space
  - **Dynamic Title Display**: Font size and line clamp scale with cover size
- **Table Display**: Tabular view with formatted author names matching filter panel
- **Selection**: Click on a book to view detailed information

#### Right Panel (Book Details)
Shows comprehensive information for the selected book:
- Book cover image
- Title and authors
- Series information (if applicable)
- Tags/categories
- Publisher and publication date
- Rating (if available)
- Comments/description

#### Bottom Panel (Status & View Controls)
- **Current library name**: Shows active library selection
- **Total number of books**: Total books in current library
- **Number of filtered books**: Books matching current filters and search
- **Current operation status**: Real-time feedback (Ready, Loading, etc.)
- **Admin button**: Visible to administrators only (🔐)
- **View Mode Toggle**: Switch between Grid and Table display
- **Cover Size Control** (Grid mode): Range slider and current size display
- **Column Visibility Controls** (Table mode): Checkboxes to show/hide table columns
- **Note**: Status panel is hidden during login for a cleaner login experience

### Admin Panel (For Administrators)

Accessible to users with admin role via the 🔐 Admin button:

#### User Management
- **Add User**: Create new user accounts with specified role and email
- **List Users**: View all users with their roles, emails, and creation dates
- **Edit User**: Update user role and email information
- **Reset Password**: Change user passwords (admin only)
- **Delete User**: Remove user accounts from the system

#### Features
- Role-based user creation (Admin, Librarian, User, Reader)
- Secure password management with Argon2id hashing
- Audit logging of all admin operations
- Admin button hidden for non-admin users
- Page protection prevents direct access for non-admins

## API Reference

### Endpoints

#### Authentication
- `POST /api/auth/login` - Login with username and password
- `POST /api/auth/logout` - Logout current user session

#### Libraries
- `GET /api/libraries` - Get list of all available libraries
- `GET /api/libraries/{id}` - Get details of a specific library

#### Books
- `GET /api/libraries/{id}/books` - Get all books in a library
- `GET /api/libraries/{id}/books/{book_id}` - Get details of a specific book
- `GET /api/libraries/{id}/books/{book_id}/cover` - Get cover image for a book

#### Metadata
- `GET /api/libraries/{id}/authors` - Get all authors in a library
- `GET /api/libraries/{id}/tags` - Get all tags in a library
- `GET /api/libraries/{id}/series` - Get all series in a library

#### Admin Endpoints (Admin role required)
- `POST /api/admin/users` - Create new user
- `GET /api/admin/users` - List all users
- `PUT /api/admin/users/{username}` - Update user role and email
- `DELETE /api/admin/users/{username}` - Delete user
- `POST /api/admin/users/{username}/password` - Reset user password

#### Query Parameters
- `search`: Filter books by title or author name
- `author_ids`: Filter by author IDs (comma-separated)
- `tag_ids`: Filter by tag IDs (comma-separated)
- `series_ids`: Filter by series IDs (comma-separated)

### Response Format

All API responses follow this format:

```json
{
  "success": true,
  "data": { /* response data */ },
  "error": null
}
```

## Configuration

### Library Path

By default, libraries are loaded from the `./libraries` directory relative to where the binary is run. To use a different path, modify the `libraries_path` in `src/main.rs`:

```rust
let libraries_path = Path::new("./libraries");  // Change this path
```

Then rebuild the application.

### Server Port

The server listens on `0.0.0.0:8080` by default. To change the port, modify `src/main.rs`:

```rust
.bind("0.0.0.0:8080")?  // Change port here
```

## Development

### Build for Development
```bash
cargo build
./target/debug/biblio
```

### Run with Logging
```bash
RUST_LOG=debug cargo run
```

### Run Tests
```bash
cargo test
```

## Database Format

Biblio reads Calibre's SQLite metadata.db files. The main tables accessed are:

- `books` - Book metadata (title, timestamp, etc.)
- `authors` - Author information
- `books_authors_link` - Join table for book-author relationships
- `tags` - Tag/category information
- `books_tags_link` - Join table for book-tag relationships
- `series` - Series information
- `books_series_link` - Join table for book-series relationships

## Performance

- **Library Scanning**: On startup, all libraries are scanned to build an in-memory cache
- **Book Loading**: Initial load of all books happens when a library is selected
- **Filtering**: All filtering is done client-side for instant response
- **Cover Images**: Served directly from Calibre's cache directory

## Browser Support

- Chrome/Chromium 90+
- Firefox 88+
- Safari 14+
- Edge 90+

## Known Limitations

1. **Filter Accuracy**: Tag/Author/Series filters currently use simplified matching
2. **Large Libraries**: Performance may degrade with libraries containing 10,000+ books
3. **Book Formats**: Only displays cover images; doesn't provide access to book files
4. **Read-Only**: Currently read-only access to Calibre libraries (modifications not supported)
5. **File-Based Storage**: User data stored in file-based format (not database backend)
6. **No Email Verification**: Email addresses stored but not verified
7. **No Password Policies**: Any password accepted (to be enforced in production)
8. **Single Admin Escalation**: System requires at least one admin account to exist

## Completed Enhancements

- [x] **User Authentication** - Secure login system with role-based access control
- [x] **Admin Panel** - User management interface for administrators
- [x] **Session Persistence** - Saves user filters, search, and library selection with cookies
- [x] **Table View** - Tabular display of books with resizable columns and visibility controls
- [x] **Dynamic Cover Sizing** - Range slider (50-250px) with automatic layout adjustment
- [x] **Responsive Titles** - Title font size and line count scale with cover size
- [x] **Auto-Fill Screen** - Automatically loads books to fill available space when resizing or switching libraries
- [x] **Search Quick Clear** - Clear button (×) for instant search field clearing
- [x] **Consistent Author Formatting** - Author names displayed consistently across grid, table, and filter views
- [x] **Smart Column Widths** - Ensures table columns have minimum readable widths
- [x] **Clean Login UI** - Status panel hidden during login for cleaner appearance

## Future Enhancements

- [ ] Advanced filtering with AND/OR logic
- [ ] Book metadata editing
- [ ] Book file access and download
- [ ] Per-user library access (fine-grained permissions)
- [ ] Dark theme
- [ ] Advanced search with full-text indexing
- [ ] Book recommendations
- [ ] Reading progress tracking
- [ ] Export/Import functionality
- [ ] Mobile app
- [ ] Email notifications for admin events
- [ ] Activity audit log viewer

## Troubleshooting

### Libraries Not Found
- Ensure the `libraries` folder exists and contains Calibre library folders
- Each library folder must contain a `metadata.db` file
- Check file permissions - the application needs read access to these files

### Cover Images Not Showing
- Ensure the `cache/covers/` subdirectory exists in your Calibre library
- Check that cover images are available in your Calibre installation
- Try opening the library in Calibre to verify it's valid

### Port Already in Use
- The default port is 8080. If it's already in use, modify `src/main.rs` and rebuild
- Or use: `lsof -i :8080` to find what's using the port

### Slow Performance
- Reduce the number of books loaded by filtering at startup
- Consider splitting large libraries into smaller ones
- Check system resources (RAM, CPU)

## License

This project is licensed under the MIT License - see the [LICENSE](MIT-License.txt) file for details.

## User Roles

Biblio supports four user roles with different permission levels:

### Admin
- Full access to all features
- User management (create, update, delete)
- Password reset for other users
- View all users list
- Access to admin panel

### Librarian
- Full access to library browsing features
- Can search, filter, and view all books
- Reserved for future library management features

### User
- Access to library browsing features
- Can search, filter, and view books
- Standard user permissions

### Reader
- Read-only access to libraries
- Can browse and search books
- Limited to viewing operations only

## Default Admin Credentials

When first deployed, an admin account is available:
- **Username**: `admin`
- **Password**: `Admin@Pass123!`

**Important**: Change this password after first login for security.

## Documentation

For detailed documentation, see the `doc/` folder:
- [ADMIN_FEATURES.md](doc/ADMIN_FEATURES.md) - Complete admin system documentation
- [IMPLEMENTATION.md](doc/IMPLEMENTATION.md) - Technical implementation details
- [AUTHENTICATION.md](doc/AUTHENTICATION.md) - Authentication system details
- [INDEX.md](doc/INDEX.md) - Documentation index and navigation guide

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Support

For issues, questions, or suggestions, please contact the development team or open an issue on the project repository.

---

**Version**: 0.3.0 (Enhanced UI/UX Release)  
**Last Updated**: January 17, 2026

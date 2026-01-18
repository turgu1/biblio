# Example: Setting Up Biblio with Sample Libraries

This guide walks through a complete example of setting up Biblio with sample Calibre libraries.

## Step 1: Verify Installation

```bash
cd <biblio-parent-folder>/biblio

# Check binary exists
ls -lh target/release/biblio
# Output: -rwxrwxr-x ... 9.3M ... biblio
```

## Step 2: Prepare Library Directory

```bash
# Create the libraries directory structure
mkdir -p libraries

# List what you have
ls -la libraries/
```

## Step 3: Copy or Create Sample Libraries

### Option A: Copy from Existing Calibre Installation

```bash
# If you have Calibre installed with libraries
ls ~/ | grep -i calibre
# Common locations:
# - ~/Calibre Library
# - ~/Library/Calibre Library (macOS)
# - ~/.local/share/calibre (Linux)

# Copy a library
cp -r ~/Calibre\ Library libraries/MyBooks

# Verify
ls libraries/MyBooks/
# Should see: metadata.db, cache/, Authors/, ...
```

### Option B: Use Multiple Libraries

```bash
# If you have multiple Calibre libraries
cp -r ~/Calibre\ Library libraries/Library1
cp -r ~/Backups/MyBooks2 libraries/Library2
cp -r ~/Documents/Research libraries/Academic

# Verify
ls -d libraries/*/
# Output:
# libraries/Library1/
# libraries/Library2/
# libraries/Academic/
```

## Step 4: Verify Library Structure

```bash
# Check the structure
tree -L 2 libraries/

# Or with ls
find libraries/ -maxdepth 2 -type f -name metadata.db

# Expected output:
# libraries/MyBooks/metadata.db
# libraries/AnotherLibrary/metadata.db
```

The structure should look like:
```
libraries/
├── MyBooks/
│   ├── metadata.db
│   ├── cache/
│   │   ├── covers/
│   │   │   ├── 1.jpg
│   │   │   ├── 2.jpg
│   │   │   └── ...
│   │   └── ...
│   ├── Authors/
│   │   ├── Author One/
│   │   │   └── Book Title/
│   │   └── ...
│   └── ...
└── AnotherLibrary/
    ├── metadata.db
    ├── cache/
    │   └── covers/
    └── ...
```

## Step 5: Configure the Application

Before starting the server, you need to set up your configuration:

```bash
cd <biblio-parent-folder>/biblio

# Copy the configuration example
cp config.yaml.example config.yaml

# Edit config.yaml with your settings
# Using your favorite editor (nano, vi, code, etc.)
nano config.yaml
```

Update these settings in `config.yaml`:

```yaml
# Set the path to your libraries directory
# Relative paths are resolved from the current working directory
# If you created libraries/ in the current directory, use:
library_path: "./libraries"

# Or use an absolute path:
# library_path: "/home/username/my-libraries"

# The IP and port for the server (default is usually fine)
service_ip_and_port: "0.0.0.0:8080"

# Path to users credentials file (optional, relative or absolute)
users_file_path: "./users.ids"

# HTTPS settings (optional, disabled by default)
use_https: false
```

**Note**: This assumes you're running in standard mode (not in Docker). 
For Docker deployment, see the README.md or set `APP_IN_DOCKER=true` and use `/config/config.yaml`.

## Step 6: Start the Server

### Option A: Using Cargo (Development)

```bash
cd <biblio-parent-folder>/biblio

# Build and run
cargo run --release

# Output:
# Compiling biblio v0.1.0
# ...
# Finished release [optimized] target(s) in X.XXs
# Running `target/release/biblio`
# Starting Biblio server on http://0.0.0.0:8080
```

### Option B: Using Pre-built Binary

```bash
cd <biblio-parent-folder>/biblio

# Run directly
./target/release/biblio

# Output:
# Starting Biblio server on http://0.0.0.0:8080
```

### Option C: Running in Background

```bash
# On Linux/macOS
./target/release/biblio &

# With nohup (survives terminal close)
nohup ./target/release/biblio > biblio.log 2>&1 &

# Check if running
ps aux | grep biblio
```

## Step 7: Access the Web Interface

Open your browser to:
```
http://localhost:8080
```

You should see:
1. Biblio header with action buttons
2. Left panel showing your libraries
3. Empty books grid with message "Select a library to view books"
4. Right panel showing "Select a book to view details"
5. Bottom status bar

## Step 8: Select and Browse a Library

1. In the **left panel**, click on a library name (e.g., "MyBooks")
2. Wait for the interface to load (1-2 seconds)
3. The **center panel** should now show book covers in a grid
4. The **bottom status bar** should show:
   - Library: MyBooks
   - Books: (number of books)
   - Filtered: (same number initially)

## Step 9: Search and Filter

### Search Books
1. In the **center panel toolbar**, type in the search box
2. Type a book title or author name
3. Grid updates instantly with matching books

### Filter by Author
1. In the **left panel**, find the "Authors" section
2. Check the checkbox next to an author name
3. Grid updates to show only books by that author
4. Check multiple authors to combine filters

### Filter by Tags/Categories
1. In the **left panel**, find the "Tags" section
2. Check desired tags
3. Grid filters automatically

### Filter by Series
1. In the **left panel**, find the "Series" section
2. Check a series name
3. Grid shows only books in that series

## Step 10: View Book Details

1. Click on any book cover in the grid
2. The book gets a blue border
3. The **right panel** shows:
   - Book cover image
   - Title and authors
   - Series (if applicable)
   - Tags
   - Publisher and publication date
   - Rating
   - Comments/description

## Step 11: Additional Features

### Sort Books
In the **center panel toolbar**:
- Recent (default) - newest books first
- Title A-Z - alphabetical by title
- Author A-Z - alphabetical by author

### Refresh Libraries
Click the **"🔄 Refresh"** button in the top panel to reload all libraries

### Clear Filters
Uncheck filter checkboxes in the left panel to show all books again

## Example Workflow

```
1. Server running: ./target/release/biblio
2. Browser open: http://localhost:8080
3. Loaded libraries shown: "MyBooks (1250)", "Academic (340)"
4. Click library "MyBooks" → Grid shows 1250 books
5. Search "Tolkien" → Grid shows 5 books
6. Click on "The Hobbit" → Right panel shows details
7. Uncheck search, check author "Asimov" → Grid shows 15 Asimov books
8. Sort by "Title A-Z" → Books alphabetically ordered
9. Clear all filters → Back to all 1250 books
10. Switch to "Academic" library → Now showing 340 academic books
```

## Troubleshooting This Setup

### Libraries Not Appearing
```bash
# Verify metadata.db exists
ls libraries/*/metadata.db

# If empty:
ls libraries/
# Should show: MyBooks/  AnotherLibrary/  ...

# If not showing:
mkdir -p libraries
cp -r ~/Calibre\ Library libraries/MyLibrary
```

### Books Grid Shows "No Books"
```bash
# Check library has books in metadata.db
sqlite3 libraries/MyBooks/metadata.db "SELECT COUNT(*) FROM books;"
# Should show a number > 0

# If 0:
# Try with a different Calibre library
```

### Covers Not Showing
```bash
# Check cache/covers directory exists
ls libraries/MyBooks/cache/covers/ | head -5
# Should show: 1.jpg, 2.jpg, 3.jpg, ...

# If empty:
# The library may not have generated covers in Calibre
# Try opening the library in Calibre and exporting with covers
```

### Server Won't Start
```bash
# Check port is not in use
lsof -i :8080

# If in use, either:
# 1. Kill other process: kill -9 <PID>
# 2. Change port in src/main.rs and rebuild

# Check permissions
ls -la <biblio-parent-folder>/biblio/target/release/biblio
# Should be executable (x permission)
```

## Performance Expectations

With the setup above:
- **Startup time**: < 1 second
- **Library load time**: < 2 seconds
- **Search response**: < 100ms
- **Filter response**: < 50ms
- **Sort response**: < 100ms
- **Memory usage**: 50-100 MB

With 1000+ books per library:
- **Grid rendering**: < 500ms
- **Scroll performance**: Smooth 60 FPS
- **Memory usage**: 80-150 MB

## Advanced Configuration

### Multiple Libraries with Symbolic Links

If your libraries are in different locations:

```bash
# Create links instead of copying
mkdir -p libraries
ln -s ~/Calibre\ Library libraries/Main
ln -s ~/Backups/OldBooks libraries/Archive
ln -s ~/Library/Calibre\ Library libraries/Secondary

# Verify
ls -la libraries/
# lrwxrwxrwx ... Main -> /home/user/Calibre Library
# lrwxrwxrwx ... Archive -> /home/user/Backups/OldBooks
```

### Systemd Service (Linux)

Create `/etc/systemd/system/biblio.service`:

```ini
[Unit]
Description=Biblio E-book Library Browser
After=network.target

[Service]
Type=simple
User=<username>
WorkingDirectory=<biblio-parent-folder>/biblio
ExecStart=<biblio-parent-folder>/biblio/target/release/biblio
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Then:
```bash
sudo systemctl daemon-reload
sudo systemctl enable biblio
sudo systemctl start biblio
sudo systemctl status biblio
```

### Nginx Reverse Proxy

Create `/etc/nginx/sites-available/biblio`:

```nginx
server {
    listen 80;
    server_name books.example.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location /api/ {
        proxy_pass http://localhost:8080/api/;
        proxy_buffering off;
    }
}
```

Enable:
```bash
sudo ln -s /etc/nginx/sites-available/biblio /etc/nginx/sites-enabled/
sudo nginx -s reload
```

## Cleanup and Maintenance

```bash
# View server logs
tail -f biblio.log

# Stop running server
killall biblio

# Clean build artifacts (if needed)
cargo clean

# Update dependencies
cargo update

# Check for issues
cargo check
cargo clippy
```

## Next Steps

Now that you have Biblio running:

1. **Explore features**: Try all search and filter options
2. **Customize appearance**: Edit colors in `public/index.html`
3. **Add more libraries**: Copy additional Calibre libraries
4. **Set up automation**: Create systemd service or cron job
5. **Remote access**: Set up reverse proxy for external access
6. **Monitoring**: Add logging and alerting

---

**Example Status**: Complete Setup ✓  
For more help, see:
- [README.md](README.md) - Full documentation
- [QUICKSTART.md](QUICKSTART.md) - Quick start guide
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - Commands and reference

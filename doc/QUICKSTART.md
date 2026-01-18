# Quick Start Guide - Biblio

Get up and running with Biblio in minutes!

## 1. Build the Application

```bash
cd <biblio-parent-folder>/biblio
cargo build --release
```

The binary will be created at: `target/release/biblio`

## 2. Configure the Application

Before running, you need to set up your configuration:

```bash
# Copy the configuration example
cp config.yaml.example config.yaml

# Edit config.yaml with your settings
# - library_path: Point to your Calibre libraries directory
# - service_ip_and_port: Server address and port (default: 0.0.0.0:8080)
# - Other settings as needed
```

## 3. Prepare Your Calibre Libraries

You need to have Calibre libraries with metadata.db files. If you don't have any yet, you can:

### Option A: Export from Calibre
1. Open Calibre
2. Select a library in the left sidebar
3. Use the Calibre GUI to locate the library folder (usually `~/Calibre Library`)
4. Copy the entire library folder to the `libraries/` directory

### Option B: Create a Sample Library (for testing)
```bash
# Create directories
mkdir -p libraries/TestLibrary/cache/covers
mkdir -p "libraries/TestLibrary/Author Name"

# Copy a Calibre metadata.db if you have one available
# or use an existing Calibre library folder
```

## 4. Directory Structure

After adding libraries, your structure should look like:

```
biblio/
├── target/
│   └── release/
│       └── biblio (the executable)
├── public/
├── libraries/
│   ├── MyLibrary/
│   │   ├── metadata.db
│   │   ├── cache/
│   │   │   └── covers/
│   │   │       ├── 1.jpg
│   │   │       ├── 2.jpg
│   │   │       └── ...
│   │   └── Author Name/
│   │       └── Book.epub
│   └── AnotherLibrary/
│       ├── metadata.db
│       └── ...
└── README.md
└── config.yaml            # Your configuration file
└── config.yaml.example    # Configuration template
```

## 5. Run the Application

```bash
# Method 1: From the project directory
cargo run --release

# Method 2: Run the binary directly
./target/release/biblio

# Method 3: Run with debug output
RUST_LOG=debug ./target/release/biblio
```

You should see output like:
```
Starting Biblio server on http://0.0.0.0:8080
```

## 6. Access the Web Interface

Open your web browser and navigate to:
```
http://localhost:8080
```

You should see the Biblio interface with your libraries loaded!

## 7. First Steps in the App

1. **Select a Library**: In the left panel, select one of your libraries
2. **View Books**: The center panel will populate with book covers
3. **Search Books**: Use the search box to find books by title or author
4. **View Details**: Click on a book to see its full details in the right panel
5. **Filter**: Use the authors, tags, and series filters in the left panel

## Keyboard Shortcuts (Future)

These shortcuts will be implemented in future versions:
- `Ctrl+F` - Focus search box
- `Ctrl+L` - Focus library selector
- `Esc` - Deselect book

## Common Tasks

### Change Server Port

Edit `src/main.rs` and find this line:
```rust
.bind("0.0.0.0:8080")?
```

Change `8080` to your desired port, then rebuild:
```bash
cargo build --release
```

### Change Libraries Directory

Edit `src/main.rs` and find this line:
```rust
let libraries_path = Path::new("./libraries");
```

Change to your preferred path, then rebuild.

### Use Existing Calibre Libraries

If you have Calibre installed and want to use your existing libraries:

```bash
# Find your Calibre library location
# Usually: ~/Calibre Library (on Linux/Mac) or C:\Users\YourName\Calibre Library (on Windows)

# Create a symbolic link or copy the library
ln -s ~/Calibre\ Library libraries/MyLibrary
# or
cp -r ~/Calibre\ Library libraries/MyLibrary

# Then run Biblio
./target/release/biblio
```

## Troubleshooting

### "No libraries found" message
- Ensure the `libraries/` directory exists
- Check that each library folder contains a `metadata.db` file
- Verify file permissions allow the application to read the files

### Can't compile the project
```bash
# Update Rust to the latest version
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Port 8080 is already in use
```bash
# Find what's using the port
lsof -i :8080

# Either:
# 1. Stop the other application
# 2. Or change the port in src/main.rs and rebuild
```

### No book covers showing
- Ensure your Calibre libraries have the `cache/covers/` directory
- Cover images should be named as `{book_id}.jpg`
- Check permissions on the cache directory

### Slow performance with large libraries
- Try filtering to reduce the number of books displayed
- Split your library into multiple smaller libraries
- Ensure you're using the `--release` build for better performance

## Next Steps

- Read the full [README.md](README.md) for complete documentation
- Check the API reference in the README for advanced usage
- Explore the web interface and try different filtering options
- Consider customizing colors and layout in `public/index.html`

## Getting Help

If you encounter issues:

1. Check the troubleshooting section in this guide
2. Review the [README.md](README.md) documentation
3. Check console logs:
   - In browser: Open Developer Tools (F12) > Console
   - In server: Look at terminal output
4. Check file permissions and library structure

---

**Enjoy browsing your e-book libraries!** 📚

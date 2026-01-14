use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::CalibreDb;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub metadata_db_path: PathBuf,
    pub book_count: usize,
}

pub struct LibraryScanner {
    base_path: PathBuf,
}

impl LibraryScanner {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        LibraryScanner {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    pub fn scan(&self) -> std::io::Result<Vec<LibraryMetadata>> {
        let mut libraries = Vec::new();

        // Scan for directories containing metadata.db
        for entry in WalkDir::new(&self.base_path)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let metadata_db = path.join("metadata.db");

            if metadata_db.exists() {
                // This is a Calibre library
                if let Ok(lib_metadata) = self.create_library_metadata(path, &metadata_db) {
                    libraries.push(lib_metadata);
                }
            }
        }

        // Sort by name
        libraries.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(libraries)
    }

    fn create_library_metadata(
        &self,
        path: &Path,
        metadata_db_path: &Path,
    ) -> std::io::Result<LibraryMetadata> {
        let library_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Create a stable UUID from the library path using UUID v5 (SHA-1 namespace hash)
        // This ensures the same path always produces the same ID
        let path_str = path.to_string_lossy();
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, path_str.as_bytes()).to_string();

        // Try to count books in the database
        let book_count = self.get_book_count(metadata_db_path).unwrap_or(0);

        Ok(LibraryMetadata {
            id,
            name: library_name,
            path: path.to_path_buf(),
            metadata_db_path: metadata_db_path.to_path_buf(),
            book_count,
        })
    }

    fn get_book_count(&self, metadata_db_path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let db = CalibreDb::open(metadata_db_path)?;
        let books = db.get_all_books()?;
        Ok(books.len())
    }
}

// In-memory cache of loaded libraries
pub struct LibraryCache {
    libraries: HashMap<String, LibraryMetadata>,
    databases: HashMap<String, CalibreDb>,
}

impl LibraryCache {
    pub fn new() -> Self {
        LibraryCache {
            libraries: HashMap::new(),
            databases: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.libraries.clear();
        self.databases.clear();
    }

    pub fn load_libraries(&mut self, base_path: &Path) -> std::io::Result<()> {
        let scanner = LibraryScanner::new(base_path);
        let libraries = scanner.scan()?;

        for lib in libraries {
            if let Ok(db) = CalibreDb::open(&lib.metadata_db_path) {
                self.databases.insert(lib.id.clone(), db);
                self.libraries.insert(lib.id.clone(), lib);
            }
        }

        Ok(())
    }

    pub fn get_libraries(&self) -> Vec<LibraryMetadata> {
        let mut libs: Vec<_> = self.libraries.values().cloned().collect();
        libs.sort_by(|a, b| a.name.cmp(&b.name));
        libs
    }

    pub fn get_library(&self, id: &str) -> Option<&LibraryMetadata> {
        self.libraries.get(id)
    }

    pub fn get_database(&self, id: &str) -> Option<&CalibreDb> {
        self.databases.get(id)
    }
}

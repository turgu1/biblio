use rusqlite::{Connection, Result as SqlResult, OptionalExtension, OpenFlags};
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub authors: Vec<String>,
    pub series: Option<String>,
    pub series_index: Option<f64>,
    pub tags: Vec<String>,
    pub comments: Option<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub rating: Option<i32>,
    pub timestamp: Option<String>,
    pub language: Option<String>,
    pub has_cover: bool,
    pub formats: Vec<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: i32,
    pub name: String,
    pub sort: Option<String>,
    pub book_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i32,
    pub name: String,
    pub book_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Series {
    pub id: i32,
    pub name: String,
    pub sort: Option<String>,
    pub book_count: i32,
}

pub struct CalibreDb {
    conn: Connection,
}

impl CalibreDb {
    pub fn open<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        Ok(CalibreDb { conn })
    }

    pub fn get_all_books(&self) -> SqlResult<Vec<Book>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, has_cover, sort FROM books ORDER BY timestamp DESC LIMIT 10000"
        )?;
        
        let books = stmt.query_map([], |row| {
            let book_id: i32 = row.get(0)?;
            let title: String = row.get(1)?;
            let has_cover: bool = row.get(2)?;
            let sort: Option<String> = row.get(3).ok();
            
            let authors = self.get_book_authors(book_id).unwrap_or_default();
            let (series, series_index) = self.get_book_series(book_id).unwrap_or_default();
            let tags = self.get_book_tags(book_id).unwrap_or_default();
            let formats = self.get_book_formats(book_id).unwrap_or_default();
            let comments = self.get_book_comments(book_id).ok().flatten();
            
            Ok(Book {
                id: book_id,
                title,
                authors,
                series,
                series_index,
                tags,
                comments,
                publisher: None,
                pubdate: None,
                rating: None,
                timestamp: None,
                language: None,
                has_cover,
                formats,
                sort,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(books)
    }

    pub fn get_book(&self, book_id: i32) -> SqlResult<Option<Book>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, has_cover, sort FROM books WHERE id = ?"
        )?;
        
        let book = stmt.query_row([book_id], |row| {
            let id: i32 = row.get(0)?;
            let title: String = row.get(1)?;
            let has_cover: bool = row.get(2)?;
            let sort: Option<String> = row.get(3).ok();
            
            let authors = self.get_book_authors(id).unwrap_or_default();
            let (series, series_index) = self.get_book_series(id).unwrap_or_default();
            let tags = self.get_book_tags(id).unwrap_or_default();
            let formats = self.get_book_formats(id).unwrap_or_default();
            let comments = self.get_book_comments(id).ok().flatten();
            
            Ok(Book {
                id,
                title,
                authors,
                series,
                series_index,
                tags,
                comments,
                publisher: None,
                pubdate: None,
                rating: None,
                timestamp: None,
                language: None,
                has_cover,
                formats,
                sort,
            })
        }).optional()?;

        Ok(book)
    }

    pub fn get_book_authors(&self, book_id: i32) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.name FROM authors a 
             INNER JOIN books_authors_link bal ON a.id = bal.author 
             WHERE bal.book = ?"
        )?;
        
        let authors = stmt.query_map([book_id], |row| {
            row.get::<_, String>(0)
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(authors)
    }

    pub fn get_book_tags(&self, book_id: i32) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.name FROM tags t 
             INNER JOIN books_tags_link btl ON t.id = btl.tag 
             WHERE btl.book = ?"
        )?;
        
        let tags = stmt.query_map([book_id], |row| {
            row.get::<_, String>(0)
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(tags)
    }

    pub fn get_book_series(&self, book_id: i32) -> SqlResult<(Option<String>, Option<f64>)> {
        let mut stmt = self.conn.prepare(
            "SELECT s.name, b.series_index FROM series s 
             INNER JOIN books_series_link bsl ON s.id = bsl.series 
             INNER JOIN books b ON bsl.book = b.id
             WHERE bsl.book = ?"
        )?;
        
        let result = stmt.query_row([book_id], |row| {
            let name: String = row.get(0)?;
            let index: f64 = row.get(1)?;
            Ok((Some(name), Some(index)))
        }).optional()?;

        Ok(result.unwrap_or((None, None)))
    }

    pub fn get_all_authors(&self) -> SqlResult<Vec<Author>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.sort, COUNT(ba.book) as book_count 
             FROM authors a 
             LEFT JOIN books_authors_link ba ON a.id = ba.author 
             GROUP BY a.id, a.name, a.sort 
             ORDER BY a.sort"
        )?;
        
        let authors = stmt.query_map([], |row| {
            Ok(Author {
                id: row.get(0)?,
                name: row.get(1)?,
                sort: row.get(2).ok(),
                book_count: row.get(3)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(authors)
    }

    pub fn get_all_tags(&self) -> SqlResult<Vec<Tag>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.name, COUNT(btl.book) as book_count 
             FROM tags t 
             LEFT JOIN books_tags_link btl ON t.id = btl.tag 
             GROUP BY t.id, t.name 
             ORDER BY t.name"
        )?;
        
        let tags = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                book_count: row.get(2)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(tags)
    }

    pub fn get_all_series(&self) -> SqlResult<Vec<Series>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.name, s.sort, COUNT(bs.book) as book_count 
             FROM series s 
             LEFT JOIN books_series_link bs ON s.id = bs.series 
             GROUP BY s.id, s.name, s.sort
             ORDER BY s.sort"
        )?;
        
        let series = stmt.query_map([], |row| {
            Ok(Series {
                id: row.get(0)?,
                name: row.get(1)?,
                sort: row.get(2).ok(),
                book_count: row.get(3)?,
            })
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(series)
    }

    pub fn get_book_formats(&self, book_id: i32) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT format FROM data WHERE book = ? ORDER BY format"
        )?;
        
        let formats = stmt.query_map([book_id], |row| {
            row.get::<_, String>(0)
        })?.collect::<SqlResult<Vec<_>>>()?;

        Ok(formats)
    }

    pub fn get_book_comments(&self, book_id: i32) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT text FROM comments WHERE book = ?"
        )?;
        
        let comments = stmt.query_row([book_id], |row| {
            row.get::<_, String>(0)
        }).optional()?;

        Ok(comments)
    }
}

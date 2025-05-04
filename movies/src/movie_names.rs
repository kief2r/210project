// Summary: Provides a movie database by loading movie titles and IDs from a CSV file.
// It allows looking up a movie’s title by its numeric ID.
// Uses a HashMap internally for fast lookups.
use std::collections::HashMap;
use std::error::Error;
use csv::Reader;
use serde::Deserialize;

// Struct Movie
// Represents one record (row) in the movies CSV.
// Fields:
// - movie_id: unique numeric ID of the movie (from the CSV column "movieId")
// - title: the title of the movie (e.g., "Real Genius (1985)")
#[derive(Debug, Deserialize)]
struct Movie {
    #[serde(rename = "movieId")]
    movie_id: u32,
    title: String,
}

// Struct MovieDb
// Represents a movie database
// Internally stores a HashMap<u32, String> that maps movie IDs to their titles
pub struct MovieDb {
    movies: HashMap<u32, String>,
}

impl MovieDb {
    // Load the movie database from a CSV file
    // Input:
    // - path: &str --> path to the CSV file containing movieId and title columns
    // Output:
    // - Result<MovieDb, Error> --> MovieDb instance on success
    // Logic:
    // - Open and parse the CSV file
    // - For each row, insert (movie_id, title) into the HashMap
    pub fn from_path(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut rdr = Reader::from_path(path)?;
        let mut movies = HashMap::new();
        for result in rdr.deserialize() {
            let rec: Movie = result?; // Deserialize each CSV row into a Movie struct
            movies.insert(rec.movie_id, rec.title); // Add to the map
        }
        Ok(MovieDb { movies })
    }

   // Get the title of a movie by its ID
    // Input:
    // - movie_id: u32 --> the ID of the movie to look up
    // Output:
    // - Option<&str> --> Some(title) if found, or None if the ID is not in the database
    pub fn get_title(&self, movie_id: u32) -> Option<&str> {
        self.movies.get(&movie_id).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: verifies that MovieDb loads correctly and returns expected titles.
    // Assumes that "movies.csv" exists in the project root and contains known IDs.
    #[test]
    fn test_get_title() -> Result<(), Box<dyn Error>> {

        let db = MovieDb::from_path("movies.csv")?;
        assert_eq!(db.get_title(171), Some("Jeffrey (1995)"));
        assert_eq!(db.get_title(1297), Some("Real Genius (1985)"));
        assert_eq!(db.get_title(5224), Some("Promises (2001)"));
        assert_eq!(db.get_title(26686), Some("Ghost Dad (1990)"));
        assert_eq!(db.get_title(88744), Some("Rise of the Planet of the Apes (2011)"));
        Ok(())
    }
}
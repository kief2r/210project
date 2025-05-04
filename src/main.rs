// Movie Recommendations
// // This program recommends movies to users based on their ratings and the ratings of similar users.
// The program also includes functions to compute centrality metrics and build user-rating vectors.
// It uses the `nalgebra` and `petgraph` libraries for mathematical operations and graph handling, respectively.

mod movie_names; // Module: loads movie titles from CSV for lookup
mod top_movies;  // Module: fetches a user’s top-rated movies

use std::collections::HashMap;
use std::error::Error;

use movie_names::MovieDb;
use top_movies::top_movies;
use nalgebra::DVector;

// Type alias: maps user IDs to their dense rating vectors
// Each vector holds the user’s ratings, with positions in alignment with movie indices
type RatingMap = HashMap<u32, DVector<f32>>;

// Struct Rating: represents a single user rating
// Fields:
// - user_id: the user who gave the rating
// - movie_id: the movie being rated
// - rating: numeric score (e.g., 1.0–5.0)
// - timestamp: when the rating was submitted (UNIX time)
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Rating {
    user_id: u32,
    movie_id: u32,
    rating: f32,
    timestamp: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Step 1: Load ratings from CSV
    let ratings = load_ratings("ratings.csv")?;

    // Step 2: Build per-user rating vectors and a popularity map for tie-breaking
    let user_vectors = build_user_vectors(&ratings);
    let popularity = build_popularity_map(&ratings);
    // Load the movie database for title lookups
    let movie_db = MovieDb::from_path("movies.csv")?;

    // Step 3: Recommend movies for a given user
    let user_id = 1;
    let top_n = 5;
    let recs = recommend_movies(user_id, &user_vectors, &ratings, &popularity, top_n);

    // Print the user’s top-rated movies
    println!("Top {} recommendations for user {}:", top_n, user_id);
    for mid in recs {
        let title = movie_db.get_title(mid).unwrap_or("<unknown>");
        println!("- {}: {}", mid, title);
    }

    // Print the user’s top-rated movies
    println!("\nUser {}’s top-rated movies:", user_id);
    for (mid, rating) in top_movies(user_id, &ratings, top_n) {
        let title = movie_db.get_title(mid).unwrap_or("<unknown>");
        println!("- {}: {} (Rating: {:.1})", mid, title, rating);
    }

    Ok(())
}

// Load ratings CSV into a vector of Rating structs
// Input: path (str) --> CSV file pat
// Output: Vec<Rating> --> loaded rating records
fn load_ratings(path: &str) -> Result<Vec<Rating>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut ratings = Vec::new();
    for rec in rdr.deserialize() {
        ratings.push(rec?);
    }
    Ok(ratings)
}

// Build rating vectors per user
// Inputs:
// - ratings: &[Rating] --> all ratings
// Output:
// - RatingMap --> maps user IDs to the rating vectors (length = number of unique movies)
// Logic:
// - Build a unique, sorted movie list (mids)
// - Create a position map (movie ID --> vector index)
// - Fill each user’s vector with their ratings at the correct positions
fn build_user_vectors(ratings: &[Rating]) -> RatingMap {
    let mut mids: Vec<u32> = ratings.iter().map(|r| r.movie_id).collect();
    mids.sort_unstable();
    mids.dedup();
    let m = mids.len();
    let pos: HashMap<u32, usize> = mids.iter().copied().enumerate().map(|(i, id)| (id, i)).collect();

    let mut map = HashMap::new();
    for r in ratings {
        let vec = map.entry(r.user_id).or_insert_with(|| DVector::from_element(m, 0.0));
        vec[pos[&r.movie_id]] = r.rating;
    }
    map
}

// Calculate cosine similarity between two rating vectors.
// Inputs:
// - a, b: &DVector<f32> --> two user rating vectors
// Output:
// - f32 --> similarity score (0.0 if either vector is zero)
// Logic:
// - Compute dot product
// - Normalize by vector magnitudes
fn cosine_similarity(a: &DVector<f32>, b: &DVector<f32>) -> f32 {
    let dot = a.dot(b);
    let na = a.norm();
    let nb = b.norm();
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na * nb) }
}

// Count the number of ratings each movie has (popularity metric)
// Input: &[Rating] --> all ratings
// Output: HashMap<u32, usize> --> movie ID --> count
// Logic:
// - For each rating, increment its movie’s counter
fn build_popularity_map(ratings: &[Rating]) -> HashMap<u32, usize> {
    let mut pop = HashMap::new();
    for r in ratings {
        *pop.entry(r.movie_id).or_default() += 1;
    }
    pop
}

// Recommend movies for a user using top-k similar users
// Inputs:
// - user_id: target user
// - uvecs: map of user rating vectors
// - ratings: all ratings
// - pop: movie popularity map
// - top_n: number of recommendations
// Output:
// - Vec<u32> --> list of top movie IDs
// Logic:
// 1. Calculate similarities to other users
// 2. Pick top-k similar users
// 3. Aggregate weighted ratings from neighbors
// 4. Predict scores, sort by score, then popularity, then movie ID
// 5. Return top-N movie IDs
fn recommend_movies(
    user_id: u32,
    uvecs: &RatingMap,
    ratings: &[Rating],
    pop: &HashMap<u32, usize>,
    top_n: usize,
) -> Vec<u32> {
    let mut mids: Vec<u32> = ratings.iter().map(|r| r.movie_id).collect();
    mids.sort_unstable();
    mids.dedup();
    let pos: HashMap<u32, usize> = mids.iter().copied().enumerate().map(|(i, id)| (id, i)).collect();
    let target = &uvecs[&user_id];

    // Step 1: Calculate similarity to all other users
    let mut sims: Vec<(u32, f32)> = uvecs.iter()
        .filter(|(&uid, _)| uid != user_id)
        .map(|(&uid, vec)| (uid, cosine_similarity(target, vec)))
        .collect();
    sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let k = 5;

    // Step 2: Aggregate weighted ratings from top-k similar users
    let mut scores: HashMap<u32, f32> = HashMap::new();
    let mut weights: HashMap<u32, f32> = HashMap::new();
    for &(uid, sim) in sims.iter().take(k) {
        for r in ratings.iter().filter(|r| r.user_id == uid) {
            if target[pos[&r.movie_id]] == 0.0 {
                *scores.entry(r.movie_id).or_default() += sim * r.rating;
                *weights.entry(r.movie_id).or_default() += sim;
            }
        }
    }

    // Step 3: Compute final predicted scores and sort
    let mut preds: Vec<(u32, f32)> = scores.into_iter()
        .filter_map(|(mid, sc)| weights.get(&mid).map(|&w| (mid, sc / w)))
        .collect();
    preds.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()
        .then_with(|| pop[&b.0].cmp(&pop[&a.0]))
        .then(a.0.cmp(&b.0)));

    preds.into_iter().take(top_n).map(|(mid, _)| mid).collect()
}
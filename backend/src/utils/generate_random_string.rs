use rand::{Rng, distributions::Alphanumeric};

/// Generates a random alphanumeric string of the specified length.
///
/// The generated string contains uppercase letters (A-Z), lowercase letters (a-z),
/// and digits (0-9). This function is cryptographically secure and suitable for
/// generating tokens, passwords, or other random identifiers.
///
/// # Arguments
///
/// * `length` - The desired length of the generated string
///
/// # Returns
///
/// A `String` containing random alphanumeric characters
///
/// # Examples
///
/// ```
/// let token = generate_random_string(32);
/// assert_eq!(token.len(), 32);
///
/// let short_code = generate_random_string(8);
/// assert_eq!(short_code.len(), 8);
/// ```
///
/// # Panics
///
/// This function will not panic under normal circumstances.
pub fn generate_random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

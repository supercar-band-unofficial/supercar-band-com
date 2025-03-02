/**
 * This contains some additional necessary sql utilities not found in sqlx.
 */

/**
 * Escapes input for LIKE clauses.
 * This may need to be modified depending on the specific SQL database implementation being used.
 */
pub fn sanitize_like_clause_value(input: &str) -> String {
    input.replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}

//! IsPalindromeCLI — user-facing binary (see crate `is_palindrome_cli`).

fn main() {
    let repo = is_palindrome_cli::find_repo_root()
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));
    let code = is_palindrome_cli::run(std::env::args().collect(), &repo);
    std::process::exit(code);
}

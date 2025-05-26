#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constant() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_banner_function() {
        // Just test that the function doesn't panic
        print_banner();
    }

    #[test]
    fn test_menu_function() {
        // Just test that the function doesn't panic
        show_menu();
    }
}
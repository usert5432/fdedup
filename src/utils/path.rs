use std::path::{Path, PathBuf};

pub fn calculate_relative_path(src : &str, dst : &str) -> String {

    let mut components_src = Path::new(src).components();
    let components_dst     = Path::new(dst).parent().unwrap().components();

    let mut result_prefix  = PathBuf::new();
    let mut result_suffix  = PathBuf::new();

    let mut common_prefix  = true;

    for chunk_dst in components_dst {

        if common_prefix {
            match components_src.next() {
                Some(chunk_src) => {
                    if chunk_dst != chunk_src {
                        result_prefix.push("..");
                        result_suffix.push(chunk_src);
                        common_prefix = false;
                    }
                },
                None => { result_prefix.push(".."); },
            };
        }
        else {
            result_prefix.push("..");
        }
    }

    for chunk_src in components_src {
        result_suffix.push(chunk_src);
    }

    result_prefix.join(result_suffix).to_str().unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_relative_path_basic() {
        assert_eq!(calculate_relative_path("/a", "/b"), "a".to_string());
    }

    #[test]
    fn test_calculate_relative_path_basic_up_tree() {
        assert_eq!(calculate_relative_path("/a", "/b/c"), "../a".to_string());
    }

    #[test]
    fn test_calculate_relative_path_basic_down_tree() {
        assert_eq!(calculate_relative_path("/a/b", "/c"), "a/b".to_string());
    }

    #[test]
    fn test_calculate_relative_path_basic_fork() {
        assert_eq!(
            calculate_relative_path("/a/b/c", "/a/d/c"), "../b/c".to_string()
        );
    }

    #[test]
    fn test_calculate_relative_path_basic_no_prefix_slash() {
        assert_eq!(calculate_relative_path("a", "b"), "a".to_string());
    }

    #[test]
    fn test_calculate_relative_path_basic_self_reference() {
        assert_eq!(calculate_relative_path("a/b/c", "a/b/c"), "c".to_string());
    }

    #[test]
    fn test_calculate_relative_path_fork() {
        assert_eq!(
            calculate_relative_path(
                "/a1/a2/a3/a4/a5/a6/a7/c",
                "/a1/a2/a3/b1/a5/a6/a7/c"
            ), "../../../../a4/a5/a6/a7/c".to_string()
        );
    }

    #[test]
    fn test_calculate_relative_path_trailing_slash() {
        assert_eq!(calculate_relative_path("/a/", "/b/"), "a".to_string());
    }

    #[test]
    #[should_panic]
    fn test_calculate_relative_path_null() {
        assert_eq!(calculate_relative_path("/", "/"), "".to_string());
    }
}


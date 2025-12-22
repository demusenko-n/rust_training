use std::collections::HashMap;

fn main() {
    println!("Hello, world!");
}

fn index_string(text: &str) -> HashMap<&str, Vec<usize>> {
    let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
    let start = text.as_ptr().addr();

    for word in text.split_whitespace() {
        let offset = word.as_ptr().addr();
        map.entry(word).or_default().push(offset - start);
    }

    return map;
}

#[test]
fn test_short() {
    let map = index_string("a b a");
    assert_eq!(map["a"], vec![0, 4]);
    assert_eq!(map["b"], vec![2]);
}

#[test]
fn test_long() {
    let map = index_string(
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    );
    assert_eq!(map["Lorem"].len(), 1);
    assert_eq!(map["dolor"].len(), 2);
    assert_eq!(map["dolore"].len(), 2);
}

#[test]
fn test_cyrillic() {
    let map = index_string(" привіт привіт");
    assert_eq!(map["привіт"], vec![1, 14]);
}

#[test]
fn test_many_whitespace() {
    let map = index_string("  a   a");
    assert_eq!(map["a"], vec![2, 6]);
}

#[test]
fn test_no_words() {
    let map = index_string("       ");
    assert_eq!(map.is_empty(), true);
}

pub fn vec_char_equal(a: &Vec<char>, b: &Vec<char>) -> bool {
    let mut result: bool = true;
    if a.len() != b.len() {
        result = false;
    } else {
        for i in 0..a.len() {
            if a[i] != b[i] {
                result = false;
                break;
            }
        }
    }
    result
}
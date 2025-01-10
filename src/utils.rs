pub fn ends_with_any(s: &String, cs: &String) -> bool {
    let checks: Vec<char> = cs.chars().collect();
    for check in checks { if s.ends_with(check) { return true; } }
    return false;
}

pub fn find_first_of_any(s: &String, cs: &String) -> usize {
    let checks: Vec<char> = cs.chars().collect();
    let mut result = usize::MAX;
    for check in checks { 
        if let Some(idx) = s.find(check) { 
            if idx < result { result = idx; }
        } 
    }
    return result;
}
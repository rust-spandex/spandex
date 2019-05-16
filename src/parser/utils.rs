//! This module contains some functions that will help us managing strings.

/// Replicates a char n times.
pub fn replicate(c: char, n: usize) -> String {
    let mut string = String::new();
    for _ in 0..n {
        string.push(c);
    }
    string
}

/// Returns true if the char at specified byte is a \n.
pub fn is_new_line(content: &str, byte: usize) -> bool {
    content.is_char_boundary(byte)
        && content.is_char_boundary(byte + 1)
        && &content[byte..byte + 1] == "\n"
}

/// Computes the column of a specified byte depending on its line and offset.
pub fn compute_column(content: &str, start: usize, current: usize) -> usize {
    let mut column = 0;
    let mut pointer = start;

    for c in content[start..].chars() {
        if pointer == current {
            break;
        }

        column += 1;
        pointer += c.len_utf8();
    }

    column
}

/// Finds the previous \n char.
///
/// Returns 0 is no \n was found.
pub fn previous_new_line(content: &str, byte: usize) -> usize {
    let mut i = byte;

    while i != 0 && !is_new_line(content, i) {
        i -= 1;
    }

    if &content[i..i + 1] == "\n" {
        i + 1
    } else {
        i
    }
}

/// Finds the next \n char.
///
/// Returns the length of the string if no \n was found.
pub fn next_new_line(content: &str, byte: usize) -> usize {
    let mut i = byte;

    while i != content.len() && !is_new_line(content, i) {
        i += 1;
    }

    i
}

//! This module contains the functions related to ligatures.

/// Ligates a string.
///
/// From https://en.wikipedia.org/wiki/List_of_precomposed_Latin_characters_in_Unicode#Digraphs_and_ligatures
pub fn ligature(input: &str) -> String {

    let mut output = String::new();

    let mut first = input.chars();
    let mut second = input.chars().skip(1);
    let mut third = input.chars().skip(2);

    loop {
        let f = first.next();
        let s = second.next();
        let t = third.next();

        match (f, s, t) {
            (Some('f'), Some('f'), Some('i')) => {
                output.push('ﬃ');
                first.next(); second.next(); third.next();
                first.next(); second.next(); third.next();
            },
            (Some('f'), Some('f'), Some('l')) => {
                output.push('ﬄ');
                first.next(); second.next(); third.next();
                first.next(); second.next(); third.next();
            },
            (Some('f'), Some('f'), _) => { output.push('ﬀ'); first.next(); second.next(); third.next(); },
            (Some('f'), Some('i'), _) => { output.push('ﬁ'); first.next(); second.next(); third.next(); },
            (Some('f'), Some('l'), _) => { output.push('ﬂ'); first.next(); second.next(); third.next(); },
            (Some('I'), Some('J'), _) => { output.push('Ĳ'); first.next(); second.next(); third.next(); },
            (Some('i'), Some('j'), _) => { output.push('ĳ'); first.next(); second.next(); third.next(); },
            (Some('L'), Some('J'), _) => { output.push('Ǉ'); first.next(); second.next(); third.next(); },
            (Some('L'), Some('j'), _) => { output.push('ǈ'); first.next(); second.next(); third.next(); },
            (Some('l'), Some('j'), _) => { output.push('ǉ'); first.next(); second.next(); third.next(); },
            (Some('N'), Some('J'), _) => { output.push('Ǌ'); first.next(); second.next(); third.next(); },
            (Some('N'), Some('j'), _) => { output.push('ǋ'); first.next(); second.next(); third.next(); },
            (Some('n'), Some('j'), _) => { output.push('ǌ'); first.next(); second.next(); third.next(); },
            (Some(c), _, _) => output.push(c),
            _ => break,
        }
    }

    output
}

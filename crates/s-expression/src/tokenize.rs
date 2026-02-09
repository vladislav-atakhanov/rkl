pub fn tokenize(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    let mut in_comment = false;

    let mut iter = input.char_indices().peekable();

    while let Some((i, c)) = iter.next() {
        // Если мы внутри комментария — пропускаем всё до конца строки
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            continue;
        }

        match c {
            ';' => {
                if let Some(&(_, ';')) = iter.peek() {
                    iter.next(); // съедаем второй ';'

                    if let Some(s) = start {
                        tokens.push(&input[s..i]);
                        start = None;
                    }

                    in_comment = true;
                } else {
                    // одиночный ; считаем символом
                    if start.is_none() {
                        start = Some(i);
                    }
                }
            }

            '(' | ')' => {
                if let Some(s) = start {
                    tokens.push(&input[s..i]);
                    start = None;
                }
                tokens.push(&input[i..i + c.len_utf8()]);
            }

            c if c.is_whitespace() => {
                if let Some(s) = start {
                    tokens.push(&input[s..i]);
                    start = None;
                }
            }

            _ => {
                if start.is_none() {
                    start = Some(i);
                }
            }
        }
    }

    if let Some(s) = start {
        tokens.push(&input[s..]);
    }

    tokens
}

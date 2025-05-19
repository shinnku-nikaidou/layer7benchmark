use rand::Rng;
use regex::Regex;

#[derive(Clone)]
enum Part {
    Literal(String),
    RandomChars { chars: Vec<char>, count: usize },
}

pub fn template_generator(template: &str) -> impl Fn() -> String + use<> {
    let re = Regex::new(r"\[([^\]]+)\](?:\{(\d+)\})?").unwrap();

    let mut parts: Vec<Part> = Vec::new();
    let mut last_end = 0;

    for caps in re.captures_iter(template) {
        let m = caps.get(0).unwrap();
        if m.start() > last_end {
            parts.push(Part::Literal(template[last_end..m.start()].to_string()));
        }
        let expr = caps.get(1).unwrap().as_str();
        let count = caps
            .get(2)
            .map_or(1, |m| m.as_str().parse::<usize>().unwrap());
        let mut chars = Vec::new();
        let mut chars_iter = expr.chars().peekable();
        while let Some(c) = chars_iter.next() {
            if let Some(&dash) = chars_iter.peek() {
                if dash == '-' {
                    chars_iter.next();
                    if let Some(end) = chars_iter.next() {
                        for ch in c..=end {
                            chars.push(ch);
                        }
                        continue;
                    }
                }
            }
            chars.push(c);
        }
        parts.push(Part::RandomChars { chars, count });
        last_end = m.end();
    }
    if last_end < template.len() {
        parts.push(Part::Literal(template[last_end..].to_string()));
    }
    move || {
        let mut rng = rand::rng();
        let mut result = String::new();
        for part in &parts {
            match part {
                Part::Literal(text) => {
                    result.push_str(text);
                }
                Part::RandomChars { chars, count } => {
                    for _ in 0..*count {
                        let idx = rng.random_range(0..chars.len());
                        result.push(chars[idx]);
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let generator =
            template_generator("https://shinnku.com/?x=[0-9]&y=[a-zA-Z]{2}&z=[a-zA-Z0-9]");
        for _ in 0..14 {
            println!("{}", generator());
        }
    }

    #[test]
    fn test2() {
        let generator = template_generator("https://shinnku.com/?x=[a-zA-Z0-9]{10}&y=[a-zA-Z]{2}");
        for _ in 0..30 {
            println!("{}", generator());
        }
    }

    #[test]
    fn test3() {
        let generator = template_generator("https://shinnku.com/[a-zA-Z0-9]{10}");
        for _ in 0..30 {
            println!("{}", generator());
        }
    }
}

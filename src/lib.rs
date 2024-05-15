use std::{process::Command, path::Path, fs};
use errors::TagNotFoundError;

mod errors;

pub fn run() {
    if !Path::new("_site").exists() {
        fs::create_dir("_site").unwrap();
    }

    let template = fs::read_to_string("template.html").unwrap();
    let other_html_files = fs::read_dir("./").unwrap();
    other_html_files
        .filter_map(Result::ok)
        .filter(|file| if let Some(e) = file.path().extension() { e == "html" } else { false })
        .filter(|file| file.file_name() != "template.html")
        .for_each(|file| {
            let file_name = file.file_name().into_string().unwrap();
            
            let raw_file = fs::read_to_string(&file_name).unwrap();
            let injected_file = replace_placeholders(template.clone(), Some(raw_file)); // inject html into template
            let fully_injected_file = replace_recursive(injected_file);

            let new_file_path = format!("_site/{}", &file_name);
            fs::write(new_file_path, fully_injected_file).unwrap();
        });

    Command::new("cp")
        .arg("-r")
        .arg("Images")
        .arg("_site")
        .spawn()
        .unwrap();

    Command::new("cp")
        .arg("style.css")
        .arg("_site")
        .spawn()
        .unwrap();
}

fn replace_recursive(mut text: String) -> String {
    while has_placeholders(&text) {
        text = replace_placeholders(text, None);
    }
    text
}

fn replace_placeholders(text: String, file_content: Option<String>) -> String {
    let mut new_text = String::new();
    for line in text.lines() {
        let mut new_line = line.to_owned();
        for word in line.split('#') {
            if word.starts_with('$') && word.ends_with('$') {
                let content = get_content(&word[1..word.len() - 1], file_content.clone());

                // append hashtags to both sides of tag to properly clean them up, as they were excluded by line.split('#')
                let mut full_tag = String::from("#");
                full_tag.push_str(word);
                full_tag.push('#');

                new_line = line.replace(&full_tag, content.as_str());
            }
        }
        new_line.push('\n');
        new_text.push_str(new_line.as_str());
    }
    new_text
}

fn has_placeholders(text: &str) -> bool {
    for line in text.lines() {
        for word in line.split('#') {
            if word.starts_with('$') && word.ends_with('$') {
                return true;
            }
        }
    }
    false
}

fn get_content(tag: &str, file_content: Option<String>) -> String {
    match tag {
        "LAST_UPDATED" => {
            let date_cmd = Command::new("date").output().unwrap();
            let date_string = String::from_utf8(date_cmd.stdout).unwrap();
            date_string.trim().to_owned()
        },
        "REAL_TAG" => String::from("other text?"),
        "RECURSIVE_TAG" => String::from("testing testing...!\n#$REAL_TAG$#"),
        "FILE_CONTENT" | "ARTICLE_CONTENT" => file_content.expect("File contents not provided"),
        no_match => panic!("{}", TagNotFoundError(no_match.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use crate::{replace_placeholders, replace_recursive};

    #[test]
    fn placeholder_insertion() {
        let text = String::from("hello world\n# nothing in particular #\nless in particular\n#$maybe???#\nbla bla\n#$REAL_TAG$#\nmore filler text\n");
        let expected = String::from("hello world\n# nothing in particular #\nless in particular\n#$maybe???#\nbla bla\nother text?\nmore filler text\n");
        let result = replace_placeholders(text, None);
        assert_eq!(result, expected);
    }

    #[test]
    fn recursive_insertion() {
        let text = String::from("hello world\n#$RECURSIVE_TAG$#\nabcdef\n");
        let expected = String::from("hello world\ntesting testing...!\nother text?\nabcdef\n");
        let result = replace_recursive(text);
        assert_eq!(result, expected);
    }

    #[test]
    fn file_insertion() {
        let text = String::from("hello world\n#$FILE_CONTENT$#\nabcdef\n");
        let expected = String::from("hello world\nexample content\nabcdef\n");
        let result = replace_placeholders(text, Some(String::from("example content")));
        assert_eq!(result, expected);
    }

    #[test]
    fn string_replace() {
        let text = "test string literal";
        let string_to_append = String::from("owned string");
        let result = text.replace("string literal", string_to_append.as_str());
        assert_eq!(result, "test owned string");
    }
}

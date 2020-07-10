use std::collections::HashMap;
use regex::{RegexSet, Regex, Captures};

/*
If owned String type doesn't work, it's because the Hash and Eq both need to return true on .get()
meaning Hash is returning something not the same as the reference in the map...
*/

#[derive(Default, Clone, Debug)]
pub struct MediaParser {
    regex_sets: HashMap<String, RegexSet>,
    regex_map: HashMap<String, Regex>,     
}

impl MediaParser {
    pub fn new(regex_sets: HashMap<String, RegexSet>, regex_map: HashMap<String, Regex>) -> Self {
        MediaParser {
            regex_sets,
            regex_map,
        }
    }

    pub fn match_set(&self, key: &str, window_title: &str) -> Option<&str> {
        let regex_set = self.regex_sets.get(key)?;
        if regex_set.is_match(window_title) {
            // println!("found matching regex set with key {} for {}", key, window_title);
            let mut matches = regex_set.matches(window_title).into_iter();
            let match_ndx = matches.next()?;
            return Some(&regex_set.patterns()[match_ndx]);
        }
        None
    }

    pub fn parse_media<'b>(&self, trimmed_title: &'b str, key: &str) -> Option<Captures<'b>> {
        // println!("attempting to parse media {} with key {}", trimmed_title, key);
        let pattern = self.match_set(key, trimmed_title)?;
        // println!("found matching media pattern: {}", pattern);
        let regex = self.regex_map.get(&pattern.to_string())?;
        // println!("returning captures group");
        regex.captures(trimmed_title)
    }

    pub fn check_and_trim_window_title<'b>(&self, window_title: &'b str, key: &str, group: &str) -> Option<&'b str> {
        let pattern = self.match_set(key, window_title)?;
        // println!("found matching pattern {} for l{}", pattern, window_title);
        self.trim_window_title(window_title, pattern, group)
    }

    fn trim_window_title<'b>(&self, window_title: &'b str, pattern: &str, group: &str) -> Option<&'b str> {
        let regex = self.regex_map.get(&pattern.to_string())?;
        let captures = regex.captures(window_title)?;
        // println!("attempting to trim {} with group {}", window_title, group);
        Some(captures.name(group)?.as_str())
    }
    
    // pub fn parse_window_title(&self, window_title: &str) -> Option<String> {
    //     if let Some(pattern) = self.match_set("player", window_title) {
    //         Some(pattern)
    //     } else if let Some(pattern) = self.match_set("browser", window_title) {
    //         if let Some(content) = self.trim_window_title(window_title, pattern, "tab") {
    //             self.parse_media(content);
    //         }
    
    //         Some(pattern)
    //     } else {
    //         None
    //     }
    // }

    pub async fn parse_window_titles(&self) -> Option<String> {
        let group = "tab";
        let player = "player";
        let browser = "browser";
    
        for title in super::get_window_titles().iter() {
            if let Some(trimmed) = self.check_and_trim_window_title(title, player, group) {
                let captures = self.parse_media(trimmed, "anime")?;
                return Some(String::from(captures.name("title")?.as_str()));
            } else if let Some(trimmed) = self.check_and_trim_window_title(title, browser, group) {
                if let Some(captures) = self.parse_media(trimmed, "anime") {
                    return Some(String::from(captures.name("title")?.as_str()));
                } else if let Some(captures) = self.parse_media(trimmed, "manga") {
                    return Some(String::from(captures.name("title")?.as_str()));
                }
            }
        }
        None
    }
}










#[cfg(test)]
mod test {
    use super::*;
    use regex::Captures;
    use std::collections::HashMap;
    use lazy_static::lazy_static;
    
    lazy_static! {
        static ref BLANKET: Regex = Regex::new(r"(?P<content>.+) - Mozilla Firefox").unwrap();
        static ref MD: Regex = Regex::new(r"(?P<content>.+) - MangaDex").unwrap();
        static ref REGEX: Regex = Regex::new(r"(?P<title>.+) - (Vol[.] (?P<volume>\d+) )?(Ch[.] (?P<chapter>(\d+[.])?\d+) )?(?P<oneshot>Oneshot)?.*?").unwrap();
    }

    fn check_title<'a>(window_title: &'a str) -> Captures<'a> {
        assert!(BLANKET.is_match(window_title));

        let title = BLANKET.captures(window_title).unwrap().name("content").unwrap().as_str();
        assert!(MD.is_match(title));

        let content = MD.captures(title).unwrap().name("content").unwrap().as_str();
        assert!(REGEX.is_match(content));

        REGEX.captures(content).unwrap()
    }

    fn test(window_title: &str, expected: HashMap<&str, &str>) {
        let cap = check_title(window_title);
        for (key, value) in expected {
            assert_eq!(value, cap.name(key).unwrap().as_str());
        }
    }

    #[test]
    fn test1() {
        let mut expected = HashMap::new();
        expected.insert("title", "Tobaku Datenroku Kaiji: 24-Oku Dasshutsu Hen");
        expected.insert("chapter", "351");
        test(
            "Tobaku Datenroku Kaiji: 24-Oku Dasshutsu Hen - Ch. 351 Intimacy - MangaDex - Mozilla Firefox",
            expected
        );
    }
    
    #[test]
    fn test2() {
        let mut expected = HashMap::new();
        expected.insert("title", "Boku no Kokoro no Yabai Yatsu");
        expected.insert("volume", "3");
        expected.insert("chapter", "39.1");
        // expected.insert("oneshot", "");
        test(
            "Boku no Kokoro no Yabai Yatsu - Vol. 3 Ch. 39.1 Extra Chapter 35 A Day Off - MangaDex - Mozilla Firefox",
            expected
        );
    }

    #[test]
    fn test3() {
        let mut expected = HashMap::new();
        expected.insert("title", "Mochi Au Lait's Short Oneshot Collection");
        // expected.insert("volume", "");
        // expected.insert("chapter", "");
        expected.insert("oneshot", "Oneshot");
        test(
            "Mochi Au Lait's Short Oneshot Collection - Oneshot - MangaDex - Mozilla Firefox",
            expected
        );
    }

    #[test]
    #[should_panic]
    fn test4() {
        test("regex - Rust - Mozilla Firefox", HashMap::new());
    }

}
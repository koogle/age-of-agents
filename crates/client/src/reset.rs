//! Platform input controls provide caret, selection, clipboard and keyboard support.

const PROMPT: &str =
    "Reset game? All progress will be lost.\nIsland seed (leave blank for a random island):";
const INVALID: &str = "Enter a whole number from 0 to 18446744073709551615, or leave blank.";

/// Cancel is None; confirmation contains either a seed or the random choice.
pub fn choose_seed() -> Option<Option<u64>> {
    let mut value = String::new();
    let mut message = PROMPT.to_owned();
    loop {
        value = input(&message, &value)?;
        match parse_seed(&value) {
            Ok(seed) => return Some(seed),
            Err(()) => message = format!("{INVALID}\n\n{PROMPT}"),
        }
    }
}

fn parse_seed(value: &str) -> Result<Option<u64>, ()> {
    let value = value.trim();
    if value.is_empty() {
        Ok(None)
    } else if value.bytes().all(|byte| byte.is_ascii_digit()) {
        value.parse().map(Some).map_err(|_| ())
    } else {
        Err(())
    }
}

#[cfg(target_arch = "wasm32")]
fn input(message: &str, value: &str) -> Option<String> {
    web_sys::window()?
        .prompt_with_message_and_default(message, value)
        .ok()
        .flatten()
}

#[cfg(not(target_arch = "wasm32"))]
fn input(message: &str, value: &str) -> Option<String> {
    tinyfiledialogs::input_box("Reset game", message, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeds_preserve_the_full_u64_range_and_blank_means_random() {
        assert_eq!(parse_seed("  "), Ok(None));
        assert_eq!(parse_seed("0"), Ok(Some(0)));
        assert_eq!(parse_seed(" 0042 "), Ok(Some(42)));
        assert_eq!(parse_seed("18446744073709551615"), Ok(Some(u64::MAX)));
        for invalid in ["18446744073709551616", "-1", "+1", "1.5", "1e3", "abc"] {
            assert_eq!(parse_seed(invalid), Err(()), "{invalid}");
        }
    }
}

use std::io;

// Copyright (c) 2015 by Shipeng Feng.

// Some rights reserved.

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:

//     * Redistributions of source code must retain the above copyright
//       notice, this list of conditions and the following disclaimer.

//     * Redistributions in binary form must reproduce the above
//       copyright notice, this list of conditions and the following
//       disclaimer in the documentation and/or other materials provided
//       with the distribution.

//     * The names of the contributors may not be used to endorse or
//       promote products derived from this software without specific
//       prior written permission.

// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


/// Prompts for confirmation (yes/no question).
///
/// - `text` - the question to ask
/// - `default` - the default for the prompt
/// - `prompt_suffix` - a suffix that should be added to the prompt
/// - `show_default` - shows or hides the default value
///
pub fn confirm(text: &str, default: bool, prompt_suffix: &str, show_default: bool) -> bool {
    let default_string = match default {
        true => Some("Y/n"),
        false => Some("y/N"),
    };
    let prompt_text = build_prompt_text(text, prompt_suffix, show_default, default_string);

    loop {
        let prompt_input = get_prompt_input(prompt_text.as_str(), false).to_ascii_lowercase();
        match prompt_input.trim() {
            "y" | "yes" => {
                return true;
            }
            "n" | "no" => {
                return false;
            }
            "" => {
                return default;
            }
            _ => {
                println!("Error: invalid input");
            }
        }
    }
}

/// Prompts a user for input.
///
/// - `text` - the text to show for the prompt.
/// - `default` - the default value to use if no input happens.
/// - `hide_input` - the input value will be hidden
/// - `confirmation` - asks for confirmation for the value
/// - `prompt_suffix` - a suffix that should be added to the prompt
/// - `show_default` - shows or hides the default value
///
pub fn prompt(
    text: &str,
    default: Option<&str>,
    hide_input: bool,
    confirmation: bool,
    prompt_suffix: &str,
    show_default: bool,
) -> String {
    let prompt_text = build_prompt_text(text, prompt_suffix, show_default, default.clone());

    let mut prompt_input: String;
    loop {
        prompt_input = get_prompt_input(prompt_text.as_str(), hide_input);
        if prompt_input != "".to_string() {
            break;
        } else if default.is_some() {
            return default.unwrap().to_string();
        }
    }

    if !confirmation {
        return prompt_input;
    }
    let mut confirm_input: String;
    loop {
        confirm_input = get_prompt_input("Repeat for confirmation: ", hide_input);
        if confirm_input != "".to_string() {
            break;
        }
    }
    if prompt_input == confirm_input {
        return prompt_input;
    } else {
        panic!("Error: the two entered values do not match");
    }
}

fn build_prompt_text(
    text: &str,
    suffix: &str,
    show_default: bool,
    default: Option<&str>,
) -> String {
    let prompt_text: String;
    if default.is_some() && show_default {
        prompt_text = format!("{} [{}]", text, default.unwrap());
    } else {
        prompt_text = text.to_string();
    }
    prompt_text + suffix
}

fn get_prompt_input(prompt_text: &str, _hide_input: bool) -> String {
    print!("{}", prompt_text);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .ok()
        .expect("Failed to read line");
    return input.trim_end_matches("\n").to_string();
}

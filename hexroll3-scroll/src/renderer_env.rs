/*
// Copyright (C) 2020-2025 Pen, Dice & Paper
//
// This program is dual-licensed under the following terms:
//
// Option 1: (Non-Commercial) GNU Affero General Public License (AGPL)
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.
//
// Option 2: Commercial License
// For commercial use, you are required to obtain a separate commercial
// license. Please contact ithai at pendicepaper.com
// for more information about commercial licensing terms.
*/
#![allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]

use caith::RollResultType;
use minijinja::Environment;
use rand::SeedableRng;
use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};

use crate::instance::SandboxInstance;

pub fn prepare_renderer(env: &mut Environment, instance: &SandboxInstance) {
    env.add_filter("bulletize", func_bulletize);
    env.add_filter("count_identical", func_count_identical);

    env.add_function("appender", func_appender);
    env.add_function("articlize", func_articlize);
    env.add_function("capitalize", func_capitalize);
    env.add_function("currency", func_currency(instance));
    env.add_function("first", func_first);
    env.add_function("float", func_float);
    env.add_function("hex_coords", func_hex_coords);
    env.add_function("if_plural_else", func_if_plural_else);
    env.add_function("int", func_int);
    env.add_function("length", func_length);
    env.add_function("list_to_obj", func_list_to_obj);
    env.add_function("max", func_max);
    env.add_function("maybe", func_maybe);
    env.add_function("plural", func_plural);
    env.add_function("plural_with_count", func_plural_with_count);
    env.add_function("round", func_round);
    env.add_function("sandbox", func_sandbox(instance));
    env.add_function("sortby", func_sortby);
    env.add_function("stable_dice", func_stable_dice);
    env.add_function("sum", func_sum);
    env.add_function("title", func_capitalize);
    env.add_function("trim", func_trim);
    env.add_function("unique", func_unique);
    env.add_function("html_link", func_html_link(instance));
    env.add_function("reroller", func_reroll);

    // unimplemented
    env.add_function("begin_spoiler", func_nop_0);
    env.add_function("end_spoiler", func_nop_0);
    env.add_function("toc_breadcrumb", func_nop_0);
    env.add_function("sandbox_breadcrumb", func_nop_0);
    env.add_function("note_button", func_nop_1);
    env.add_function("note_container", func_nop_1);
}

fn func_bulletize(value: Vec<String>, seperator: &str) -> String {
    value.join(&format!(" &#{}; ", seperator)).to_string()
}

fn func_articlize(
    value: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    if let Some(noun) = value.as_str() {
        fn is_plural(noun: &str) -> bool {
            noun.ends_with('s') && noun != "bus" && noun != "grass" && noun != "kiss"
        }
        fn starts_with_vowel_sound(word: &str) -> bool {
            let vowels = ["a", "e", "i", "o", "u"];
            if let Some(first_char) = word.chars().next() {
                vowels.contains(&first_char.to_lowercase().to_string().as_str())
            } else {
                false
            }
        }
        let article = if is_plural(noun) {
            return Ok(String::from(noun));
        } else if starts_with_vowel_sound(noun) {
            "an"
        } else {
            "a"
        };
        Ok(format!("{} {}", article, noun))
    } else {
        Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "Function articlize received a non-string value",
        ))
    }
}

fn func_capitalize(
    possible_str: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<minijinja::value::Value, minijinja::Error> {
    if let Some(v) = possible_str.as_str() {
        let mut v = v.to_string();
        if !v.is_empty() {
            v[0..1].make_ascii_uppercase(); // Capitalize the first character
        }
        Ok(minijinja::Value::from(v))
    } else {
        let t: serde_json::Value = possible_str.clone();
        Ok(minijinja::value::Value::from_serialize(&t))
    }
}

fn func_currency(
    _instance: &SandboxInstance,
) -> impl Fn(minijinja::value::ViaDeserialize<serde_json::Value>) -> Result<String, minijinja::Error>
{
    let currency_factor = 1.0;
    move |v: minijinja::value::ViaDeserialize<serde_json::Value>| -> Result<String, minijinja::Error> {
        if let Some(v) = v.as_f64() {
            let v = v * currency_factor;
            if v > 1.0 {
                return Ok(format!("{} gp", format_with_commas(v as i64)));
            }
            if v > 0.1 {
                return Ok(format!("{:.0} sp", (v * 10.0).round() as i64));
            }
            if v > 0.01 {
                return Ok(format!("{:.0} cp", (v * 100.0).round() as i64));
            }
            return Ok(format!("{:.0} gp", v as i64));
        }
        Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "Currency value is not floating point",
        ))
    }
}

fn func_first(
    possible_str: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<minijinja::value::Value, minijinja::Error> {
    if let Some(v) = possible_str.as_array() {
        if let Some(first) = v.iter().next() {
            return Ok(minijinja::Value::from_serialize(first.clone()));
        }
    } else if let Some(v) = possible_str.as_str() {
        if let Some(first) = v.chars().next() {
            return Ok(minijinja::Value::from_serialize(first));
        }
    }
    Err(minijinja::Error::new(
        minijinja::ErrorKind::UndefinedError,
        "func_first could not pick the first item from an array",
    ))
}

fn func_float(value: &str) -> Result<f32, minijinja::Error> {
    if let Ok(value) = value.trim().parse::<f32>() {
        Ok(value)
    } else {
        Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "Unable to convert value in func_float",
        ))
    }
}

fn func_int(value: &str) -> Result<i64, minijinja::Error> {
    if let Ok(value) = value.trim().parse::<i64>() {
        Ok(value)
    } else {
        Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "Unable to convert value in func_int",
        ))
    }
}

fn func_length(
    c: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<usize, minijinja::Error> {
    if let Some(r) = c.as_array() {
        Ok(r.len())
    } else {
        Ok(0)
    }
}

fn func_list_to_obj(
    list: minijinja::value::ViaDeserialize<serde_json::Value>,
    attr_name: &str,
) -> Result<minijinja::value::Value, minijinja::Error> {
    let mut map = serde_json::json!({});
    if let Some(list) = list.as_array() {
        for item in list.iter() {
            if let Some(key) = item.as_object().unwrap().get(attr_name) {
                if let Some(key_str) = key.as_str() {
                    let m = &mut map.as_object_mut().unwrap();
                    if !m.contains_key(key_str) {
                        m.insert(key_str.to_string(), serde_json::json!([]));
                    }
                    m[key_str].as_array_mut().unwrap().push(item.clone());
                }
            }
        }
    }
    Ok(minijinja::value::Value::from_serialize(map))
}

fn func_count_identical(list: Vec<String>) -> HashMap<String, i32> {
    let mut counts = HashMap::new();
    for item in list {
        *counts.entry(item).or_insert(0) += 1;
    }
    counts
}

fn func_trim(
    _c: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    if let Some(value) = _c.as_str() {
        return Ok(clean_string(value.to_string()));
    }
    Err(minijinja::Error::new(
        minijinja::ErrorKind::UndefinedError,
        "Function trim did not get a string",
    ))
}

fn func_sortby(
    list: minijinja::value::ViaDeserialize<serde_json::Value>,
    attr_to_sortby: &str,
) -> Result<minijinja::value::Value, minijinja::Error> {
    let mut ret = serde_json::json!([]);
    if let Some(list) = list.as_array() {
        let mut list_to_sort = list.clone();
        list_to_sort.sort_by(|a, b| {
            let a_value = a.get(attr_to_sortby).and_then(|v| v.as_str()).unwrap_or("");
            let b_value = b.get(attr_to_sortby).and_then(|v| v.as_str()).unwrap_or("");
            a_value.cmp(b_value)
        });
        ret = serde_json::Value::Array(list_to_sort.to_vec());
    }
    Ok(minijinja::value::Value::from_serialize(ret))
}

fn func_hex_coords(
    _: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    Ok(String::from("TBD"))
}

fn func_maybe(
    v: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    if let Some(s) = v.as_str() {
        return Ok(s.to_string());
    }
    Ok(String::new())
}

fn func_sandbox(instance: &SandboxInstance) -> impl Fn() -> Result<String, minijinja::Error> {
    let sid = match instance.sid.as_ref() {
        Some(sid) => sid.clone(),
        None => "".to_string(),
    };
    move || -> Result<String, minijinja::Error> { Ok(format!("/inspect/{}", sid)) }
}

fn func_round(value: f32, _dec: f32) -> Result<f32, minijinja::Error> {
    let y = (value * 100.0).round() / 100.0;
    if false {
        return Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "",
        ));
    }
    Ok(y)
}

fn func_max(a: i32, b: i32) -> Result<i32, minijinja::Error> {
    if false {
        return Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "",
        ));
    }
    Ok(max(a, b))
}

fn func_appender(parent_uid: &str, attr: &str, cls: &str) -> String {
    format!(
        r#"
        <a href="/append/{parent_uid}/{attr}/{cls}">⊞</a>
        "#
    )
}

fn func_plural(count: f32, v: &str) -> Result<String, minijinja::Error> {
    if count <= 1.0 {
        return Ok(v.to_string());
    }
    let mut plural = v.to_string();
    let c = v.chars().last().unwrap_or('\0');
    let c_minus_1 = v.chars().rev().nth(1).unwrap_or('\0');

    if "sxzh".contains(c) {
        plural.push_str("es");
    } else if c == 'y' {
        if "aeiou".contains(c_minus_1) {
            plural.push('s');
        } else {
            plural.pop();
            plural.push_str("ies");
        }
    } else if v.ends_with("olf") {
        plural.pop();
        plural.push_str("ves");
    } else {
        plural.push('s');
    }
    Ok(plural)
}

fn func_plural_with_count(count: f32, v: &str) -> Result<String, minijinja::Error> {
    if count <= 1.0 {
        return Ok(v.to_string());
    }
    Ok(format!(
        "{} {}",
        count as i32,
        func_plural(count, v).unwrap()
    ))
}

fn func_if_plural_else(
    check: &str,
    ifplural: &str,
    ifnotplural: &str,
) -> Result<String, minijinja::Error> {
    let check = check.to_lowercase();
    if check.ends_with('s') || check == "teeth" || check == "wolves" {
        Ok(ifplural.to_string())
    } else {
        Ok(ifnotplural.to_string())
    }
}

fn func_sum(l: minijinja::value::ViaDeserialize<serde_json::Value>) -> f64 {
    let mut sum = 0.0;
    for v in l.as_array().unwrap() {
        if let Ok(a) = v.as_str().unwrap().parse::<f64>() {
            sum += a;
        }
    }
    sum
}

fn func_unique(
    v: minijinja::value::ViaDeserialize<serde_json::Value>,
    attr: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<minijinja::value::Value, minijinja::Error> {
    let mut ret = serde_json::json!([]);
    let mut unique_set = HashSet::new();

    if let Some(v) = v.as_array() {
        for e in v.iter() {
            if let Some(value) = e.as_object().unwrap().get(attr.as_str().unwrap()) {
                if !unique_set.contains(value) {
                    ret.as_array_mut().unwrap().push(e.clone());
                    unique_set.insert(value.clone());
                }
            }
        }
        return Ok(minijinja::value::Value::from_serialize(&ret));
    }
    Ok(minijinja::value::Value::from_serialize(serde_json::json!(
        {}
    )))
}

fn func_stable_dice(roll: &str, uid: &str, index: u64) -> Result<i32, minijinja::Error> {
    if false {
        return Err(minijinja::Error::new(
            minijinja::ErrorKind::UndefinedError,
            "",
        ));
    }
    let roller = caith::Roller::new(roll).unwrap();
    let seed = string_to_seed(uid) + index;
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    if let RollResultType::Single(value) = roller.roll_with(&mut rng).unwrap().get_result() {
        return Ok(value.get_total() as i32);
    }
    Ok(0)
}

fn func_html_link(
    instance: &SandboxInstance,
) -> impl Fn(&str, &str) -> Result<String, minijinja::Error> {
    let sid = match instance.sid.as_ref() {
        Some(sid) => sid.clone(),
        None => "".to_string(),
    };
    move |uid, text| -> Result<String, minijinja::Error> {
        Ok(format!(
            "<a href='/inspect/{}/entity/{}'>{}</a>",
            sid, uid, text
        ))
    }
}

fn func_nop_0() -> Result<String, minijinja::Error> {
    Ok(String::new())
}

fn func_nop_1(
    _: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    Ok(String::new())
}

fn func_reroll(
    uid: minijinja::value::ViaDeserialize<serde_json::Value>,
    _: minijinja::value::ViaDeserialize<serde_json::Value>,
    _: minijinja::value::ViaDeserialize<serde_json::Value>,
) -> Result<String, minijinja::Error> {
    // "<a href='/reroll/{}'>⬣</a>",
    let id = if let Some(obj) = uid.get("uuid") {
        obj.to_string().trim_matches('"').to_string()
    } else {
        uid.to_string().trim_matches('"').to_string()
    };
    Ok(format!(
        "<a href='/reroll/{}'>⟳</a><a href='/unroll/{}'>🗑</a>",
        id, id
    )
    .to_string())
}

fn clean_string(mut s: String) -> String {
    s = s.trim().to_string();
    s.retain(|c| c != '\n' && c != '\r');
    s = s
        .chars()
        .fold((String::new(), None), |(mut acc, prev_char), c| {
            if c == ' ' && prev_char == Some(' ') {
                (acc, prev_char)
            } else {
                acc.push(c);
                (acc, Some(c))
            }
        })
        .0;
    s
}

fn format_with_commas(v: i64) -> String {
    let s = v.to_string();
    let mut formatted = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            formatted.push(',');
            count = 0;
        }
        formatted.push(c);
        count += 1;
    }
    formatted.chars().rev().collect()
}

fn string_to_seed<S: AsRef<str>>(seed_str: S) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    std::hash::Hash::hash(&seed_str.as_ref(), &mut hasher);
    std::hash::Hasher::finish(&hasher)
}

#[cfg(test)]
mod tests {}

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
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf; // Trait that provides the `choose` method

use anyhow::anyhow;
use anyhow::Result;
use minijinja::Environment;
use rand::distributions::Alphanumeric;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use crate::generators::roll;
use crate::parser::parse_buffer;
use crate::parser::parse_file;
use crate::renderer_env::prepare_renderer;
use crate::repository::*;
use crate::semantics::*;

/// SandboxBuilder is a wrapper for sandbox instances, providing the
/// additional facilities required to generate content.
pub struct SandboxBuilder<'a> {
    pub sandbox: &'a SandboxInstance,
    pub randomizer: Randomizer,
    pub templating_env: Environment<'a>,
}

impl<'a> SandboxBuilder<'a> {
    pub fn from_instance(instance: &'a SandboxInstance) -> Self {
        let mut env = Environment::new();
        prepare_renderer(&mut env, instance);
        SandboxBuilder {
            sandbox: instance,
            randomizer: Randomizer::new(),
            templating_env: env,
        }
    }
}

/// SandboxInstance holds all the data needed to read and render
/// generated content as well as the model for generating content.
pub struct SandboxInstance {
    pub sid: Option<String>,
    pub classes: HashMap<String, Class>,
    pub repo: Repository,
    pub globals: HashMap<String, serde_json::Value>,
}

impl SandboxInstance {
    pub fn new() -> Self {
        SandboxInstance {
            sid: None,
            classes: HashMap::new(),
            repo: Repository::new(),
            globals: HashMap::new(),
        }
    }

    pub fn with_scroll(&mut self, scroll_filepath: PathBuf) -> Result<&mut Self> {
        parse_file(self, scroll_filepath)?;
        Ok(self)
    }

    pub fn open(&mut self, filepath: &str) -> Result<&mut Self> {
        self.repo.open(filepath)?;
        let root = self.repo.inspect(|tx| tx.load("root"))?;
        if let Some(sid) = root.value.as_str() {
            self.sid = Some(sid.to_string());
            Ok(self)
        } else {
            Err(anyhow!("Unable to find root entity in {}", filepath))
        }
    }

    pub fn create(&mut self, filepath: &str) -> Result<&mut Self> {
        self.repo.create(filepath)?;

        if let Ok(sid) = self.repo.mutate(|tx| {
            let builder = SandboxBuilder::from_instance(self);
            let ret = roll(&builder, tx, "main", "root", None);
            tx.store("root", &serde_json::json!(ret.as_ref().unwrap()))?;
            ret
        }) {
            self.sid = Some(sid.to_string());
            Ok(self)
        } else {
            Err(anyhow!(
                "Was unable to create a new sandbox in {}",
                filepath
            ))
        }
    }

    pub fn sid(&self) -> Option<String> {
        self.sid.clone()
    }

    pub fn parse_buffer(&mut self, buffer: &str) -> &mut Self {
        parse_buffer(self, buffer, None, None).unwrap();
        self
    }
}

impl Default for SandboxInstance {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Randomizer {
    rng: RefCell<ThreadRng>, // Use RefCell for interior mutability
}

impl Randomizer {
    pub fn new() -> Self {
        Randomizer {
            rng: RefCell::new(thread_rng()),
        }
    }

    pub fn choose<'a, T>(&self, v: &'a Vec<T>) -> &'a T {
        match v.choose(&mut *self.rng.borrow_mut()) {
            Some(item) => item,
            None => panic!("List is empty"),
        }
    }

    pub fn uid(&self) -> String {
        let mut rng = self.rng.borrow_mut();
        (0..8).map(|_| rng.sample(Alphanumeric) as char).collect()
    }

    pub fn in_range(&self, min: i32, max: i32) -> i32 {
        let mut rng = self.rng.borrow_mut();
        rng.gen_range(min..max + 1)
    }
}

impl Default for Randomizer {
    fn default() -> Self {
        Self::new()
    }
}

//! For analyzing the data

use std::{collections::HashMap, fs, ops::AddAssign};
use crate::LOG_FILE;

// CONSTS
const IGNORED_CLIENT_IDS_FILE: &str = "ignored_client_ids.txt";
const MAX_RESPONSE_INTERVAL: u64 = 4;// Min difference between response timestamps to still be considered looking at the page

/// Main analysis fuction
pub fn main() {
	let dataset = Dataset::load();
	println!("{}", dataset.stats());
	println!("Individual client stats:\n{}", dataset.individual_client_stats());
}

fn load_and_parse_lines() -> Vec<Line> {
	let raw_file = fs::read_to_string(LOG_FILE).unwrap();
	let ignored_ids = load_ignored_client_ids();
	let mut out = Vec::<Line>::new();
	for line_raw in raw_file.split('\n') {
		if line_raw.len() == 1 {continue;}
		let line_parsed = Line::parse(line_raw.to_owned());
		// Check if client ID is ignored
		if let Line::Valid(entry) = &line_parsed {
			if ignored_ids.contains(&entry.client_id) {
				continue;
			}
		}
		out.push(line_parsed);
	}
	// Done
	out
}

fn load_ignored_client_ids() -> Vec<u128> {
	let raw_file = fs::read_to_string(IGNORED_CLIENT_IDS_FILE).unwrap();
	let mut out = Vec::<u128>::new();
	for (line_n_from_0, line_raw) in raw_file.split('\n').enumerate() {
		let comment_sep_tokens: Vec<&str> = line_raw.split("#").collect();
		if comment_sep_tokens.len() > 0 && comment_sep_tokens[0].len() > 1 {
			out.push(comment_sep_tokens[0].parse::<u128>().expect(&format!("Unable to parse ignored IDs file line {}", line_n_from_0 + 1)));
		}
	}
	// Done
	out
}

struct Dataset {
	clients: HashMap<u128, Client>,
	invalid_lines: Vec<String>
}

impl Dataset {
	pub fn load() -> Self {
		let lines = load_and_parse_lines();
		let mut invalid_lines = Vec::<String>::new();
		let mut clients = HashMap::<u128, Client>::new();
		for line in lines {
			match line {
				Line::Valid(entry) => match clients.get_mut(&entry.client_id) {
					Some(client) => client.entries.push(entry),
					None => {clients.insert(entry.client_id, Client{entries: vec![entry]});}
				},
				Line::Invalid(content) => invalid_lines.push(content)
			}
		}
		// Done
		Self {
			clients,
			invalid_lines
		}
	}
	pub fn stats(&self) -> String {
		let mut focus_times = Vec::<u32>::new();
		for (_, client) in &self.clients {
			focus_times.push(client.focus_time(MAX_RESPONSE_INTERVAL));
		}
		format!(
			"Number of respondants: {}\nIn-focus time (secs): {} (mean), {} (median)\nNumber of invalid response lines: {}",
			self.clients.len(),
			mean(&focus_times),
			median(&focus_times),
			self.invalid_lines.len()
		)
	}
	pub fn individual_client_stats(&self) -> String {
		let mut out = String::new();
		for (id, client) in &self.clients {
			out.push_str(&format!("\t{}: In-focus time (secs): {}\n", id, client.focus_time(MAX_RESPONSE_INTERVAL)));
		}
		// Done
		out
	}
}

struct Client {
	pub entries: Vec<ValidEntry>
}

impl Client {
	pub fn focus_time(&self, max_response_interval: u64) -> u32 {
		let mut total: u64 = 0;
		let mut focus_begin_t: u64 = 0;
		let mut prev_focus = false;
		let mut most_recent_focus_t: u64 = 0;
		let mut prev_t: u64 = 0;
		let curr_last_entry: &ValidEntry = &self.entries[self.entries.len() - 1];
		let last_entry: ValidEntry = ValidEntry::new(curr_last_entry.ts + 1, curr_last_entry.client_id, 0);// Add this on the end so any streak including the actual last entry will be counted
		for entry in self.entries.iter().chain(vec![last_entry].iter()) {
			// Record current focus
			let curr_focus = entry.status != 0_u8;
			// If focus changes
			match (prev_focus, curr_focus) {
				(false, false) => {},
				(false, true) => {
					focus_begin_t = entry.ts;
				},
				(true, false) => {
					total += most_recent_focus_t + 1 - focus_begin_t;
				}
				(true, true) => {
					// Check if difference between records is too big
					if entry.ts - prev_t > max_response_interval {
						// Add previous streak to total
						total += most_recent_focus_t + 1 - focus_begin_t;
						// Begin new streak
						focus_begin_t = entry.ts;
					}
				}
			}
			// Update times
			if curr_focus {
				most_recent_focus_t = entry.ts;
			}
			prev_focus = curr_focus;
			prev_t = entry.ts;
		}
		// Done
		total as u32
	}
}

struct ValidEntry {
	pub ts: u64,
	pub client_id: u128,
	pub status: u8
}

impl ValidEntry {
	pub fn new(
		ts: u64,
		client_id: u128,
		status: u8
	) -> Self {
		Self{ts, client_id, status}
	}
}
enum Line{
	Valid(ValidEntry),
	Invalid(String)
}

impl Line {
	pub fn parse(raw: String) -> Self {
		let tokens: Vec<&str> = raw.split(" ").collect();
		if tokens.len() == 3 {
			let ts_res = tokens[0].parse::<u64>();
			let client_id_res = tokens[1].parse::<u128>();
			let status_res = tokens[2].parse::<u8>();
			if ts_res.is_ok() && client_id_res.is_ok() && status_res.is_ok() {
				Self::Valid(ValidEntry::new(ts_res.unwrap(), client_id_res.unwrap(), status_res.unwrap()))
			}
			else {
				Self::Invalid(raw)
			}
		}
		else {
			Self::Invalid(raw)
		}
	}
}

fn sum<T>(in_: &Vec<T>) -> T where T: AddAssign + From<u8> + Copy {
	let mut out: T = 0_u8.into();
	for n in in_ {
		out += *n;
	}
	// Done
	out
}

fn mean(in_: &Vec<u32>) -> f32 {
	assert!(in_.len() > 0);
	let sum: u32 = sum(in_);
	let sum_f: f32 = sum as f32;
	sum_f / (in_.len() as f32)
}

fn median(in_: &Vec<u32>) -> f32 {// TODO: check
	assert!(in_.len() > 0);
	let mut in_sorted: Vec<u32> = in_.clone();
	in_sorted.sort();
	//dbg!(&in_);
	let i_lower: usize = (in_.len() -1 ) / 2;
	//dbg!(i_lower);
	if in_.len() % 2 == 0 && in_.len() > 1 {
		(in_sorted[i_lower] + in_sorted[i_lower + 1]) as f32 / 2.0
	}
	else {
		in_sorted[i_lower] as f32
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn focus_time_simple() {
		let client = Client {
			entries: vec![
				ValidEntry::new(0, 0, 0),
				ValidEntry::new(1, 0, 1),
				ValidEntry::new(2, 0, 1),
				ValidEntry::new(4, 0, 1),// I meant to increment the TS by 2
				ValidEntry::new(5, 0, 0),
			]
		};
		assert_eq!(client.focus_time(4), 4);
	}
	#[test]
	fn focus_time_ends() {// whether focus during beginning and end will be counted
		let client = Client {
			entries: vec![
				ValidEntry::new(1, 0, 1),
				ValidEntry::new(2, 0, 1),
				ValidEntry::new(3, 0, 1),
				ValidEntry::new(4, 0, 0),
				ValidEntry::new(5, 0, 0),
				ValidEntry::new(6, 0, 1),
				ValidEntry::new(7, 0, 1),
			]
		};
		assert_eq!(client.focus_time(4), 5);
	}
	#[test]
	fn focus_time_streak_break() {
		let client = Client {// Should count as 1 through 6, then 16 through 18
			entries: vec![
				ValidEntry::new(0, 0, 0),
				ValidEntry::new(1, 0, 1),
				ValidEntry::new(2, 0, 1),
				ValidEntry::new(4, 0, 1),// I meant to increment the TS by 2
				ValidEntry::new(5, 0, 1),
				ValidEntry::new(6, 0, 1),
				// Big time gap, shouldn't count
				ValidEntry::new(16, 0, 1),
				ValidEntry::new(17, 0, 1),
				ValidEntry::new(18, 0, 1),
				ValidEntry::new(19, 0, 0),
			]
		};
		assert_eq!(client.focus_time(4), 9);
	}
}
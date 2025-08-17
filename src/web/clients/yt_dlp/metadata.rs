use core::fmt::Display;

use serde::{Deserialize, Serialize};

mod de {
	use serde::Deserialize;
	use serde::de::Deserializer;

	pub fn string_empty_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
	where
		D: Deserializer<'de>,
	{
		let opt = Option::<String>::deserialize(deserializer)?;
		Ok(opt.filter(|s| !s.trim().is_empty()))
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct VideoInfo {
	pub title: String,
	pub duration: Option<f64>,
	#[serde(default, deserialize_with = "de::string_empty_as_none")]
	pub description: Option<String>,
	#[serde(default)]
	pub tags: Vec<String>,
	#[serde(default)]
	pub thumbnails: Vec<Thumbnail>,
	#[serde(default)]
	pub chapters: Vec<Chapter>,
	#[serde(default)]
	pub heatmap: Vec<HeatmapPoint>,
	#[serde(default, deserialize_with = "de::string_empty_as_none")]
	pub channel_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Thumbnail {
	pub url: String,
	pub id: Option<String>,
	pub width: Option<u32>,
	pub height: Option<u32>,
	pub preference: Option<i32>,
	pub resolution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Chapter {
	pub title: Option<String>,
	pub start_time: f64,
	pub end_time: Option<f64>,
}

impl Display for Chapter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let start = self.start_time.to_string();
		let end = self
			.end_time
			.map_or_else(|| "end".to_string(), |x| x.to_string());
		let title = self
			.title
			.as_deref()
			.unwrap_or("Unnamed Chapter");

		write!(f, "{title} - {start}–{end}")
	}
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HeatmapPoint {
	pub start_time: f64,
	pub end_time: f64,
	pub value: f64,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn displays_title_start_and_end() {
		let chapter = Chapter {
			title: Some("Garfield".into()),
			start_time: 12.3,
			end_time: Some(15.0),
		};
		assert_eq!(chapter.to_string(), "Garfield - 12.3–15");
	}

	#[test]
	fn displays_missing_title() {
		let chapter = Chapter {
			title: None,
			start_time: 5.0,
			end_time: Some(7.5),
		};
		assert_eq!(chapter.to_string(), "Unnamed Chapter - 5–7.5");
	}

	#[test]
	fn displays_missing_end() {
		let chapter = Chapter {
			title: Some("Odie".into()),
			start_time: 15.0,
			end_time: None,
		};
		assert_eq!(chapter.to_string(), "Odie - 15–end");
	}

	#[test]
	fn displays_missing_title_and_end() {
		let chapter = Chapter {
			title: None,
			start_time: 12.34,
			end_time: None,
		};
		assert_eq!(chapter.to_string(), "Unnamed Chapter - 12.34–end");
	}

	#[test]
	fn displays_zero_length_range() {
		let chapter = Chapter {
			title: Some("Jon bakes lasagna".into()),
			start_time: 42.0,
			end_time: Some(42.0),
		};
		assert_eq!(chapter.to_string(), "Jon bakes lasagna - 42–42");
	}

	#[test]
	fn trims_trailing_zeros() {
		let chapter = Chapter {
			title: Some("Garfield".into()),
			start_time: 1.500,
			end_time: Some(2.0),
		};
		assert_eq!(chapter.to_string(), "Garfield - 1.5–2");
	}
}

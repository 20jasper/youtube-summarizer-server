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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HeatmapPoint {
	pub start_time: f64,
	pub end_time: f64,
	pub value: f64,
}

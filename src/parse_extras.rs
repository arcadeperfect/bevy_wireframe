use serde_json::Value;
use std::collections::HashMap;

#[derive(serde::Deserialize, Debug)]
pub struct JsonLineList {
    pub line_list: Vec<[u32; 2]>,
}

pub fn parse_gltf_extra_json(json_str: &str) -> Option<HashMap<String, JsonLineList>> {
    serde_json::from_str::<Value>(json_str)
        .ok()
        .and_then(|json_value| {
            json_value
                .get("gltf_all_selected_edges")
                .and_then(|edges_str| {
                    serde_json::from_str::<Value>(edges_str.as_str()?)
                        .ok()
                        .map(|edges_json| {
                            let mut result = HashMap::new();
                            if let Some(edges_obj) = edges_json.as_object() {
                                for (key, value) in edges_obj {
                                    if let Some(edge_array) = value.as_array() {
                                        let line_list: Vec<[u32; 2]> = edge_array
                                            .iter()
                                            .filter_map(|pair| {
                                                if let Some(pair_array) = pair.as_array() {
                                                    if pair_array.len() == 2 {
                                                        Some([
                                                            pair_array[0].as_u64()? as u32,
                                                            pair_array[1].as_u64()? as u32,
                                                        ])
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        result.insert(key.clone(), JsonLineList { line_list });
                                    }
                                }
                            }
                            result
                        })
                })
        })
}

pub fn parse_selected_edges(
    json_str: &str,
) -> Result<HashMap<String, JsonLineList>, serde_json::Error> {
    let parsed: Value = serde_json::from_str(json_str)?;
    let mut result = HashMap::new();

    if let Value::Object(obj) = parsed {
        if let Some(Value::String(edges_str)) = obj.get("gltf_all_selected_edges") {
            let edges: HashMap<String, Vec<[u32; 2]>> = serde_json::from_str(edges_str)?;
            for (key, value) in edges {
                result.insert(key, JsonLineList { line_list: value });
            }
        }
    }

    Ok(result)
}

impl From<Vec<Vec<i32>>> for JsonLineList {
    fn from(edges: Vec<Vec<i32>>) -> Self {
        let line_list = edges
            .into_iter()
            .map(|e| [e[0] as u32, e[1] as u32])
            .collect();
        JsonLineList { line_list }
    }
}
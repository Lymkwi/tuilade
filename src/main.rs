//! I3 Show Tree

use serde_derive::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, convert::TryFrom, io::prelude::*};

mod utils;

enum BorderType {
    Pixel,
    None
}

impl TryFrom<&Value> for BorderType {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::String(st) => {
                match st.as_str() {
                    "pixel" => Ok(BorderType::Pixel),
                    "none" => Ok(BorderType::None),
                    _ => Err(format!("Unknown border type \"{st}\""))
                }
            }
            _ => Err(String::from("Incompatible JSON value type")),
        }
    }
}

impl ToString for BorderType {
    fn to_string(&self) -> String {
        match self {
            BorderType::Pixel => "pixel",
            BorderType::None => "none"
        }.into()
    }
}

#[derive(Deserialize)]
enum FloatMode {
    AutoOff,
    UserOn,
}

impl TryFrom<&Value> for FloatMode {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::String(st) => match st.as_str() {
                "auto_off" => Ok(FloatMode::AutoOff),
                "user_on" => Ok(FloatMode::UserOn),
                _ => Err(format!("Unknown floating type \"{st}\"")),
            },
            _ => Err(String::from("Incompatible JSON value type")),
        }
    }
}

impl ToString for FloatMode {
    fn to_string(&self) -> String {
        match self {
            FloatMode::AutoOff => "Auto Off",
            FloatMode::UserOn => "User On"
        }.into()
    }
}

enum Layout {
    Tabbed,
    SplitV,
    SplitH,
    Stacked,
}

impl TryFrom<&Value> for Layout {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        let st = utils::try_string(val)?;
        match st {
            "tabbed" => Ok(Layout::Tabbed),
            "splitv" => Ok(Layout::SplitV),
            "splith" => Ok(Layout::SplitH),
            "stacked" => Ok(Layout::Stacked),
            _ => Err(format!("Unknown layout \"{st}\"")),
        }
    }
}

impl ToString for Layout {
    fn to_string(&self) -> String {
        match self {
            Self::Tabbed => "tabbed",
            Self::SplitV => "splitv",
            Self::SplitH => "splith",
            Self::Stacked => "stacked",
        }.into()
    }
}

enum TreeType {
    Con,
}

impl ToString for TreeType {
    fn to_string(&self) -> String {
        match self {
            TreeType::Con => "con"
        }.into()
    }
}

impl TryFrom<&Value> for TreeType {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        let st = utils::try_string(val)?;
        match st {
            "con" => Ok(TreeType::Con),
            _ => Err(String::from("Unknown tree type \"{st}\"")),
        }
    }
}

#[derive(Deserialize)]
struct TreeGeometry {
    height: u64,
    width: u64,
    x: u64,
    y: u64,
}

impl TryFrom<&Value> for TreeGeometry {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        let mut fields = HashMap::new();
        for field in &["height", "width", "x", "y"] {
            let field_ser = val
                .get(field)
                .ok_or_else(|| format!("Missing field \"{field}\""))?;
            let val = utils::try_u64(field_ser)?;
            fields.insert(*field, val);
        }

        Ok(TreeGeometry {
            height: fields["height"],
            width: fields["width"],
            x: fields["x"],
            y: fields["y"],
        })
    }
}

impl TreeGeometry {
    fn pretty_print(&self) -> String {
        format!("Geometry | {{ {{ Width: {width} | Height: {height} }} | {{ X: {x} | Y: {y} }} }}",
                width=self.width,
                height=self.height,
                x=self.x,
                y=self.y
        )
    }
}

struct Node {
    border: BorderType,
    floating: FloatMode,
    layout: Option<Layout>,
    marks: Vec<String>,
    percent: f64,
    tree_type: TreeType,
    current_border_width: Option<usize>,
    nodes: Vec<Box<Node>>,
    geometry: Option<TreeGeometry>,
    name: Option<String>,
    swallows: HashMap<String, String>,
}

impl TryFrom<&Value> for Node {
    type Error = String;

    fn try_from(val: &Value) -> Result<Self, Self::Error> {
        match val {
            Value::Object(obj) => {
                // Try and get the fields
                let border_serialized = obj
                    .get("border")
                    .ok_or_else(|| String::from("Missing \"border\" field"))?;
                let border: BorderType = BorderType::try_from(border_serialized)?;

                let float_serialized = obj
                    .get("floating")
                    .ok_or_else(|| String::from("Missing \"floating\" field"))?;
                let floating = FloatMode::try_from(float_serialized)?;

                let marks_serialized = obj
                    .get("marks")
                    .ok_or_else(|| String::from("Missing \"marks\" field"))?;
                let marks: Vec<String> = match marks_serialized {
                    Value::Array(proposed_vec) => {
                        /* verify the insides of the vector */
                        proposed_vec
                            .into_iter()
                            .map(|v| match v {
                                Value::String(st) => Ok(st.clone()),
                                _ => Err(String::from("Marks contain non-strings")),
                            })
                            .collect::<Result<Vec<String>, String>>()
                    }
                    _ => Err(String::from("Invalid JSON value type for marks")),
                }?;

                let percent_ser = 
                    obj.get("percent")
                        .ok_or_else(|| String::from("Missing \"percent\" field"))?;
                let percent = match percent_ser {
                    Value::Null => 0.0_f64,
                    v => utils::try_f64(v)?
                };

                // Type
                let tree_type_serialized = obj
                    .get("type")
                    .ok_or_else(|| String::from("Missing \"type\" field"))?;
                let tree_type = TreeType::try_from(tree_type_serialized)?;

                // Layout is optional
                let layout = if let Some(ly) = obj.get("layout") {
                    Some(Layout::try_from(ly)?)
                } else {
                    None
                };

                // Name is optional
                let name = if let Some(v) = obj.get("name") {
                    Some(utils::try_string(v)?.to_owned())
                } else {
                    None
                };

                // Geometry is optional and exclusive with layout
                let geometry = if let Some(v) = obj.get("geometry") {
                    Some(
                        serde_json::from_value::<TreeGeometry>(v.clone())
                            .map_err(|e| format!("JSON Error: {e}"))?,
                    )
                } else {
                    None
                };

                // Nodes is the children...
                let nodes = if let Some(v) = obj.get("nodes") {
                    let vec = utils::try_vec(v)?;
                    if vec.iter().any(|item| !matches!(item, Value::Object(_))) {
                        return Err(String::from("Nodes contains non-objects"));
                    }

                    // Build the children
                    vec.into_iter()
                        .map(|i| {
                            let res = Node::try_from(i)?;
                            Ok(Box::new(res))
                        })
                        .collect::<Result<Vec<Box<Node>>, String>>()?
                } else {
                    Vec::new()
                };

                // Swallows is optional
                // But when it's not there, do an empty map
                let swallows = if let Some(v) = obj.get("swallows") {
                    if !v.is_array() {
                        Err(String::from("swallows is non-array"))
                    } else {
                        let arr = v.as_array().unwrap();
                        let arr = arr.into_iter()
                            .flat_map(|ent| {
                                if !ent.is_object() {
                                    vec![Err(String::from("non-object in swallows"))]
                                } else {
                                    ent.as_object().unwrap()
                                        .into_iter()
                                        .map(|(key, val)| {
                                            if !val.is_string() {
                                                Err(format!("Key \"{key}\" has non-string value"))
                                            } else {
                                                Ok((key.clone(), val.as_str().unwrap().to_owned()))
                                            }
                                        })
                                        .collect::<Vec<Result<(String, String), String>>>()
                                    }
                            })
                            .collect::<Result<Vec<(String, String)>, String>>()?;
                        Ok(arr.into_iter()
                            .collect::<HashMap<String, String>>())
                    }
                } else {
                    Ok(HashMap::new())
                }?;

                Ok(Node {
                    border,
                    floating,
                    marks,
                    layout,
                    percent,
                    tree_type,
                    name,
                    geometry,
                    nodes,

                    current_border_width: None,
                    swallows,
                })
            }
            _ => Err(String::from("Incompatible JSON value type")),
        }
    }
}

impl Node {
    fn pretty_print(&self, id: &str) -> String {
        // Ok, start formatting:
        // +-------------------------------------------------------+
        // | <NAME>Name of the Window (truncated of course)        |
        // +-------------------------------------------------------+
        // | Layout / Geometry    | Percent   |   Current Border   |
        // |----------------------|           |       Width        |
        // | Tree Type | Floating |---------+-----------+----------|
        // |----------------------+  Nodes  | Swallows  |  Marks   |
        // | Border Type          |         |           |          |
        // +--------------------------------+----------------------+

        // Build the label
        let default = "(no name)".to_owned();
        let label = format!("{{<NAME>{name}|{{ {{ {{ Tree Type:\\n{tree_type} | Floating:\\n{floating} }} | Border Type:\\n{border_type} | {lygeom} }}| {{ {{ Percent:\\n{percent:0.3}% | Border Width:\\n{cbwidth} }} | {{ {swallows} | {marks} }} }} }} }}",
            name = self.name.as_ref()
                .unwrap_or(&default)
                .replace('\\', "\\\\")
                .replace('\"', "\\\"")
                .replace('|', "\\|")
                .replace('^', "\\^")
                .replace('/', "\\/"),
            tree_type = self.tree_type.to_string(),
            floating = self.floating.to_string(),
            border_type = self.border.to_string(),
            percent = self.percent,
            lygeom = if let Some(ly) = self.layout.as_ref() {
                format!("<NODES>Layout:\\n{}", ly.to_string())
            } else {
                self.geometry.as_ref().unwrap().pretty_print()
            },
            cbwidth = self.current_border_width
                .map(|e| format!("{e}"))
                .unwrap_or("N/A".into()),
            marks = if self.marks.len() == 0 {
                "No marks".into()
            } else {
                format!("Marks:\\n{}",
                        self.marks.iter()
                            .map(|i| format!("- \\\"{i}\\\"\\l"))
                            .collect::<Vec<String>>().join(""))
            },
            swallows = if self.swallows.is_empty() {
                "No swallows"
            } else {
                "<SWALLOWS>Swallows"
            }
        );
        let mut node_itself = format!("\tnode_{id} [shape=record label=\"{label}\"]\n");
        if !self.swallows.is_empty() {
            // Build the swallows
            let the_swallows = format!("\tnode_{id}_swallows [shape=record label=\"{{ <HEAD>Swallows | {} }}\"]\n\tnode_{id}:SWALLOWS -> node_{id}_swallows:HEAD",
                self.swallows.iter()
                    .map(|(key, val)| {
                        format!("- {key}: \\\"{}\\\"\\l", val)
                    }).collect::<Vec<String>>().join(""));
            node_itself.push_str(&the_swallows);
        }

        // Children
        for (pos, child) in self.nodes.iter().enumerate() {
            // Compute the new id
            let child_id = format!("{}_{}", id, pos);
            node_itself.push_str(&child.pretty_print(&child_id));

            node_itself.push_str(&format!("\tnode_{}:NODES -> node_{}:NAME\n", id, child_id));
        }

        node_itself
    }
}

fn read_input() -> Result<String, String> {
    let mut buffer = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buffer)
        .map_err(|e| format!("I/O Error: \"{e}\""))?;
    let decoded =
        String::from_utf8(buffer).map_err(|e| format!("Unicode decoding error: \"{e}\""))?;
    Ok(decoded)
}

fn main() -> Result<(), String> {
    let code = read_input()?;
    // Destroy the first two lines
    if code == "" {
        return Ok(());
    }

    println!("digraph tuilade {{");
    println!("\tnode_title[shape=rectangle label = \"Tuilade i3 viewer\"]");
    let mut root_id = 0;
    for window in code.trim().split("\n\n") {
        let mp: serde_json::Value =
            serde_json::from_str(&window).map_err(|e| format!("JSON parse error: \"{e}\""))?;
        let mp = Node::try_from(&mp)?;
        println!("{}", mp.pretty_print(&format!("{root_id}")));
        root_id += 1;
    }
    println!("}}");
    Ok(())
}

use pipewire::{registry::GlobalObject, types::ObjectType};
use regex::Regex;

// === Non-PW

pub fn string_matches_any_pattern(s: String, patterns: &Vec<Regex>) -> bool {
    for p in patterns {
        if p.is_match(&s) {
            return true;
        }
    }
    return false;
}

pub fn patterns_to_regexes(patterns: &Vec<String>) -> Vec<Regex> {
    return patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
}

// === PW-related

pub fn pw_node_matches_regexes(
    node: &pipewire::registry::GlobalObject<&libspa::utils::dict::DictRef>,
    patterns: &Vec<Regex>,
) -> bool {
    if let Some(props) = node.props {
        if let Some(application_name) = props.get("application.name") {
            if string_matches_any_pattern(application_name.to_string(), patterns) {
                return true;
            }
        } else {
            // TODO fallback to other props
            println!("node has no application.name! {:?}", props);
        }

        return false;
    } else {
        panic!("PW Node has no props!");
    }
}

pub fn pw_node_is_readable(node: &GlobalObject<&libspa::utils::dict::DictRef>) -> bool {
    if let Some(props) = node.props {
        let media_class = props.get("media.class");
        //println!("media.class: {:?}", media_class);
        match media_class {
            Some("Stream/Output/Audio") => {
                return true;
            }
            Some(_s) => {
                //println!("media.class not recognized {:?}", _s);
                return false;
            }
            None => {
                return false;
            }
        }
    }
    return false;
}

pub fn pw_object_is_node(go: &GlobalObject<&libspa::utils::dict::DictRef>) -> bool {
    return go.type_ == pipewire::types::ObjectType::Node;
}

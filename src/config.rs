use crate::input::{Binding, Event, Mapping, Modifier};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub rule: Vec<RuleConfig>,
}

#[derive(Deserialize)]
pub struct RuleConfig {
    #[serde(default)]
    pub window_class: Vec<String>,
    pub device: Option<String>,
    pub remap: Vec<RemapConfig>,
}

#[derive(Deserialize)]
pub struct RemapConfig {
    #[serde(default)]
    pub modifier: Modifier,
    pub input: Event,
    pub output: Event,
}

impl RuleConfig {
    pub fn mappings(&self) -> Vec<Mapping> {
        self.remap
            .iter()
            .filter(|r| {
                if !r.output.is_button() {
                    eprintln!(
                        "warning: scroll->scroll remap ignored ({:?} -> {:?})",
                        r.input, r.output
                    );
                    return false;
                }
                true
            })
            .map(|r| Mapping {
                binding: Binding {
                    modifier: r.modifier,
                    input: r.input,
                },
                output: r.output,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_config() {
        let toml = r#"
            [[rule]]

            [[rule.remap]]
            modifier = "ctrl"
            input = "scroll_up"
            output = "mouse_left"

            [[rule.remap]]
            input = "scroll_down"
            output = "mouse_right"
        "#;

        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.rule.len(), 1);
        assert!(config.rule[0].window_class.is_empty());

        let mappings = config.rule[0].mappings();
        assert_eq!(mappings.len(), 2);

        assert_eq!(mappings[0].binding.input, Event::ScrollUp);
        assert_eq!(mappings[0].binding.modifier, Modifier::Ctrl);
        assert_eq!(mappings[0].output, Event::MouseLeft);
        assert_eq!(mappings[1].binding.modifier, Modifier::None);
        assert_eq!(mappings[1].output, Event::MouseRight);
    }
}

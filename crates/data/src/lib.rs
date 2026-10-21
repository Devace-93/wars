#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::manual_is_multiple_of,
    clippy::field_reassign_with_default,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::if_same_then_else,
    clippy::needless_range_loop,
    clippy::option_map_unit_fn,
    clippy::comparison_chain,
    clippy::match_like_matches_macro
)]
//! wars-data: the content tables (objects, technologies, civilisations).
//!
//! Numbers follow the classic age-advancing RTS closely (see docs/research) with the
//! simplifications listed in docs/DESIGN.md.

mod civs;
mod objects;
mod techs;

use std::sync::OnceLock;
use wars_sim::types::GameData;
use wars_sim::Fixed;

/// Build the complete game data. Panics if a cross reference is broken (tests catch it).
pub fn build() -> GameData {
    let mut d = GameData::default();
    d.objects = objects::objects();
    d.techs = techs::techs();
    d.civs = civs::civs();
    d.gather_rates = objects::gather_rates();
    d.carry_capacity = 10;
    d.base_pop_cap = 200;
    d.market_fee_pct = 30;
    d.relic_gold_per_minute = 30;
    d.wonder_countdown_secs = 600;
    d.relic_countdown_secs = 600;
    d.finish();
    let errs = d.validate();
    if !errs.is_empty() {
        panic!("game data errors:\n{}", errs.join("\n"));
    }
    d
}

/// Process-wide shared data.
pub fn data() -> &'static GameData {
    static DATA: OnceLock<GameData> = OnceLock::new();
    DATA.get_or_init(build)
}

pub(crate) fn f(s: &str) -> Fixed {
    Fixed::from_decimal(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wars_sim::types::ObjectClass;

    #[test]
    fn data_is_consistent() {
        let d = build();
        assert!(d.civs.len() >= 13);
        assert!(d.objects.len() >= 90, "objects: {}", d.objects.len());
        assert!(d.techs.len() >= 80, "techs: {}", d.techs.len());
        // Every trainable unit has a building that trains it.
        for o in &d.objects {
            if o.class == ObjectClass::Unit && o.cost.total() > 0 {
                assert!(o.trained_at.is_some(), "{} has no trainer", o.key);
            }
        }
        // Every civ's unique unit is trained at the castle.
        for c in &d.civs {
            let uu = d.object(d.obj(&c.unique_unit));
            assert_eq!(uu.trained_at.as_deref(), Some("castle"), "{}", c.key);
        }
    }
}

//! The catalog: every section and level, in course order.

use std::fmt;

use crate::engine::LevelId;
use crate::engine::level::Level;
use crate::engine::section::Section;

/// A level id that is not in the catalog. A stale saved id must never be mistaken for "the
/// player finished", so asking for the neighbours of one is an error, not `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownLevelError(pub LevelId);

impl fmt::Display for UnknownLevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown level: {}", self.0)
    }
}

impl std::error::Error for UnknownLevelError {}

/// All sections and levels. Iteration, [`Catalog::next`] and [`Catalog::prev`] follow the
/// course order, which is numeric on `(section, number)`, whatever order the sections and
/// levels were declared in.
#[derive(Debug, Clone)]
pub struct Catalog {
    sections: Vec<Section>,
    /// `(section index, level index)` for every level, sorted by id (stable).
    order: Vec<(usize, usize)>,
}

impl Catalog {
    /// Collect sections into a catalog. Nothing is rejected here (a malformed catalog is
    /// [`Catalog::validate`]'s to report), so building one never panics.
    pub fn new(mut sections: Vec<Section>) -> Self {
        sections.sort_by_key(|section| section.number);
        let mut order: Vec<(usize, usize)> = sections
            .iter()
            .enumerate()
            .flat_map(|(s, section)| (0..section.levels.len()).map(move |l| (s, l)))
            .collect();
        order.sort_by_key(|&(s, l)| sections[s].levels[l].id);
        Self { sections, order }
    }

    fn entry(&self, position: usize) -> (&Section, &Level) {
        let (s, l) = self.order[position];
        (&self.sections[s], &self.sections[s].levels[l])
    }

    fn id_at(&self, position: usize) -> LevelId {
        self.entry(position).1.id
    }

    /// Position of the first level with this id, or where it would be.
    fn position_of(&self, id: LevelId) -> usize {
        self.order
            .partition_point(|&(s, l)| self.sections[s].levels[l].id < id)
    }

    /// The number of levels.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// True when there are no levels.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Every level with its section, in course order.
    pub fn iter(&self) -> impl Iterator<Item = (&Section, &Level)> + '_ {
        (0..self.order.len()).map(|position| self.entry(position))
    }

    /// The ids of all levels, in course order.
    pub fn ids(&self) -> impl Iterator<Item = LevelId> + '_ {
        self.iter().map(|(_, level)| level.id)
    }

    /// The sections, ordered by number.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// The section with this number.
    pub fn section(&self, number: u16) -> Option<&Section> {
        self.sections
            .iter()
            .find(|section| section.number == number)
    }

    /// The level with this id and the section it belongs to.
    pub fn get(&self, id: LevelId) -> Option<(&Section, &Level)> {
        let position = self.position_of(id);
        (position < self.order.len() && self.id_at(position) == id).then(|| self.entry(position))
    }

    /// The first level of the course.
    pub fn first(&self) -> Option<LevelId> {
        (!self.is_empty()).then(|| self.id_at(0))
    }

    /// The last level of the course.
    pub fn last(&self) -> Option<LevelId> {
        self.len()
            .checked_sub(1)
            .map(|position| self.id_at(position))
    }

    /// The level after `id`, or `Ok(None)` when `id` is the last one.
    pub fn next(&self, id: LevelId) -> Result<Option<LevelId>, UnknownLevelError> {
        let position = self.known_position(id)?;
        Ok((position + 1..self.len())
            .map(|p| self.id_at(p))
            .find(|&candidate| candidate != id))
    }

    /// The level before `id`, or `Ok(None)` when `id` is the first one.
    pub fn prev(&self, id: LevelId) -> Result<Option<LevelId>, UnknownLevelError> {
        let position = self.known_position(id)?;
        Ok((0..position)
            .rev()
            .map(|p| self.id_at(p))
            .find(|&candidate| candidate != id))
    }

    fn known_position(&self, id: LevelId) -> Result<usize, UnknownLevelError> {
        let position = self.position_of(id);
        if position < self.len() && self.id_at(position) == id {
            Ok(position)
        } else {
            Err(UnknownLevelError(id))
        }
    }

    /// Map a possibly stale saved id onto a level that exists.
    ///
    /// Levels get renumbered between versions, which would otherwise leave a saved game
    /// permanently unplayable. Resolution keeps the player as close as possible to where
    /// they were: the level itself if it exists, otherwise the nearest *later* level, and
    /// the last level when the id is past the end. `None` only for an empty catalog.
    pub fn resolve_or_nearest(&self, id: LevelId) -> Option<LevelId> {
        if self.is_empty() {
            return None;
        }
        let position = self.position_of(id).min(self.len() - 1);
        Some(self.id_at(position))
    }

    /// [`Catalog::resolve_or_nearest`] for a saved id that may not even parse: a malformed
    /// one resolves to the first level.
    pub fn resolve_str(&self, raw: &str) -> Option<LevelId> {
        match raw.parse::<LevelId>() {
            Ok(id) => self.resolve_or_nearest(id),
            Err(_) => self.first(),
        }
    }

    /// Everything wrong with the catalog, as human-readable lines that start with the level
    /// id (or `section N`). Empty means the catalog is sound.
    ///
    /// Structure: unique ids, intro if and only if number 0, every regular level has an
    /// answer or a requirement and a solution. Paths: every declared path is a plain
    /// relative path that does not restate its section root, fixtures do not target their
    /// root. Pairing: `cd_evidence` needs a `cd` policy, `fd_evidence` needs `on_fd`.
    /// Content: the answer never appears in error, mistake or format messages, a `secret`
    /// answer never appears in hints or as `shellgame submit <answer>`, custom `at`
    /// messages do not name their target, no hint is empty, and every `accepts` /
    /// `rejects` example behaves as declared.
    ///
    /// Debug builds run this at startup, so a malformed level fails on `cargo run`.
    pub fn validate(&self) -> Vec<String> {
        crate::engine::lint::lint(self)
    }
}

impl<'a> IntoIterator for &'a Catalog {
    type Item = (&'a Section, &'a Level);
    type IntoIter = Box<dyn Iterator<Item = (&'a Section, &'a Level)> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::level::{intro, level};

    fn lvl(number: u16) -> Level {
        level(number, format!("level {number}"))
    }

    fn sample() -> Catalog {
        // Declared out of order on purpose.
        Catalog::new(vec![
            Section::new(10, "level-10").levels([lvl(4), lvl(1)]),
            Section::new(1, "level-1")
                .levels([lvl(12), lvl(10), lvl(2)])
                .intro(intro("Navigace", "text")),
            Section::new(2, "level-2").levels([lvl(1)]),
        ])
    }

    fn id(section: u16, number: u16) -> LevelId {
        LevelId::new(section, number)
    }

    #[test]
    fn iterates_in_numeric_course_order() {
        let catalog = sample();
        let ids: Vec<String> = catalog.ids().map(|i| i.to_string()).collect();
        assert_eq!(ids, ["1.0", "1.2", "1.10", "1.12", "2.1", "10.1", "10.4"]);
        assert_eq!(catalog.len(), 7);
        assert!(!catalog.is_empty());
        let via_iter: Vec<LevelId> = catalog.iter().map(|(_, level)| level.id).collect();
        assert_eq!(via_iter, catalog.ids().collect::<Vec<_>>());
        let via_into: Vec<LevelId> = (&catalog).into_iter().map(|(_, level)| level.id).collect();
        assert_eq!(via_into, via_iter);
    }

    #[test]
    fn sections_are_sorted_and_findable() {
        let catalog = sample();
        let numbers: Vec<u16> = catalog.sections().iter().map(|s| s.number).collect();
        assert_eq!(numbers, [1, 2, 10]);
        assert_eq!(catalog.section(2).unwrap().root, "level-2");
        assert!(catalog.section(3).is_none());
    }

    #[test]
    fn get_returns_the_level_with_its_section() {
        let catalog = sample();
        let (section, level) = catalog.get(id(1, 10)).unwrap();
        assert_eq!(section.number, 1);
        assert_eq!(level.title, "level 10");
        assert!(catalog.get(id(1, 11)).is_none(), "the 1.11 gap stays a gap");
        assert!(catalog.get(id(5, 1)).is_none());
        assert!(catalog.get(id(1, 0)).unwrap().1.is_intro());
    }

    #[test]
    fn first_and_last() {
        let catalog = sample();
        assert_eq!(catalog.first(), Some(id(1, 0)));
        assert_eq!(catalog.last(), Some(id(10, 4)));
        let empty = Catalog::new(vec![]);
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn next_follows_the_course_and_ends_with_none() {
        let catalog = sample();
        assert_eq!(catalog.next(id(1, 0)), Ok(Some(id(1, 2))));
        assert_eq!(
            catalog.next(id(1, 2)),
            Ok(Some(id(1, 10))),
            "1.10 follows 1.2, not 1.1x"
        );
        assert_eq!(
            catalog.next(id(1, 12)),
            Ok(Some(id(2, 1))),
            "across sections"
        );
        assert_eq!(catalog.next(id(10, 1)), Ok(Some(id(10, 4))));
        assert_eq!(
            catalog.next(id(10, 4)),
            Ok(None),
            "the last level has no next"
        );
    }

    #[test]
    fn next_and_prev_refuse_an_unknown_id() {
        let catalog = sample();
        assert_eq!(catalog.next(id(1, 11)), Err(UnknownLevelError(id(1, 11))));
        assert_eq!(catalog.prev(id(9, 9)), Err(UnknownLevelError(id(9, 9))));
        assert_eq!(
            Catalog::new(vec![]).next(id(1, 1)),
            Err(UnknownLevelError(id(1, 1)))
        );
        assert_eq!(
            UnknownLevelError(id(1, 11)).to_string(),
            "unknown level: 1.11"
        );
    }

    #[test]
    fn prev_mirrors_next() {
        let catalog = sample();
        assert_eq!(catalog.prev(id(1, 0)), Ok(None));
        assert_eq!(catalog.prev(id(1, 10)), Ok(Some(id(1, 2))));
        assert_eq!(catalog.prev(id(2, 1)), Ok(Some(id(1, 12))));
        assert_eq!(catalog.prev(id(10, 4)), Ok(Some(id(10, 1))));
        for window in catalog.ids().collect::<Vec<_>>().windows(2) {
            assert_eq!(catalog.next(window[0]), Ok(Some(window[1])));
            assert_eq!(catalog.prev(window[1]), Ok(Some(window[0])));
        }
    }

    #[test]
    fn resolve_keeps_an_existing_id() {
        let catalog = sample();
        for id in catalog.ids().collect::<Vec<_>>() {
            assert_eq!(catalog.resolve_or_nearest(id), Some(id));
        }
    }

    #[test]
    fn resolve_maps_a_stale_id_to_the_nearest_later_level() {
        let catalog = sample();
        assert_eq!(
            catalog.resolve_or_nearest(id(1, 11)),
            Some(id(1, 12)),
            "a gap"
        );
        assert_eq!(catalog.resolve_or_nearest(id(1, 3)), Some(id(1, 10)));
        assert_eq!(
            catalog.resolve_or_nearest(id(1, 99)),
            Some(id(2, 1)),
            "past a section's end"
        );
        assert_eq!(
            catalog.resolve_or_nearest(id(3, 1)),
            Some(id(10, 1)),
            "a missing section"
        );
        assert_eq!(
            catalog.resolve_or_nearest(id(0, 5)),
            Some(id(1, 0)),
            "before the start"
        );
    }

    #[test]
    fn resolve_past_the_end_lands_on_the_last_level() {
        let catalog = sample();
        assert_eq!(catalog.resolve_or_nearest(id(10, 5)), Some(id(10, 4)));
        assert_eq!(catalog.resolve_or_nearest(id(99, 1)), Some(id(10, 4)));
    }

    #[test]
    fn resolve_in_an_empty_catalog_is_none() {
        let empty = Catalog::new(vec![]);
        assert_eq!(empty.resolve_or_nearest(id(1, 1)), None);
        assert_eq!(empty.resolve_str("1.1"), None);
        assert_eq!(empty.resolve_str("garbage"), None);
    }

    #[test]
    fn resolve_str_handles_malformed_saved_ids_like_python() {
        let catalog = sample();
        assert_eq!(catalog.resolve_str("1.10"), Some(id(1, 10)));
        assert_eq!(catalog.resolve_str("1.11"), Some(id(1, 12)));
        assert_eq!(catalog.resolve_str("01.002"), Some(id(1, 2)));
        for bad in ["", "garbage", "1", "1.2.3", "a.b", "-1.2"] {
            assert_eq!(
                catalog.resolve_str(bad),
                Some(id(1, 0)),
                "{bad:?} falls back to the first level"
            );
        }
    }

    #[test]
    fn duplicates_do_not_break_navigation() {
        let catalog = Catalog::new(vec![Section::new(1, "level-1").levels([
            lvl(1),
            lvl(1),
            lvl(2),
        ])]);
        assert_eq!(catalog.len(), 3);
        assert_eq!(catalog.next(id(1, 1)), Ok(Some(id(1, 2))));
        assert_eq!(catalog.prev(id(1, 2)), Ok(Some(id(1, 1))));
        assert_eq!(catalog.prev(id(1, 1)), Ok(None));
        assert!(catalog.get(id(1, 1)).is_some());
    }
}

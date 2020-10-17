use super::Entry;
use crate::lang::en::{get_month_name, get_ordinal};
use crate::lang::SentenceCaseTransformer;
use crate::types::{
    EntryType, EntryTypeModality, EntryTypeSpec, NumOrStr, Person, PersonRole,
};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct ApaBibliographyGenerator<'s> {
    entries: &'s [Entry],
    formatter: SentenceCaseTransformer,
}

#[derive(Clone, Debug)]
enum SourceType<'s> {
    PeriodicalItem(&'s Entry),
    CollectionItem(&'s Entry),
    TvSeries(&'s Entry),
    Thesis,
    Manuscript,
    ArtContainer(&'s Entry),
    StandaloneArt,
    StandaloneWebItem,
    WebItem(&'s Entry),
    NewsItem(&'s Entry),
    ConferenceTalk(&'s Entry),
    Generic,
}

impl<'s> SourceType<'s> {
    fn for_entry(entry: &'s Entry) -> Self {
        let periodical = EntryTypeSpec::single_parent(
            EntryTypeModality::Any,
            EntryTypeModality::Specific(EntryType::Periodical),
        );
        let collection_a = EntryTypeSpec::single_parent(
            EntryTypeModality::Specific(EntryType::InAnthology),
            EntryTypeModality::Specific(EntryType::Anthology),
        );
        let collection_b = EntryTypeSpec::single_parent(
            EntryTypeModality::Specific(EntryType::Entry),
            EntryTypeModality::Any,
        );
        let collection_c = EntryTypeSpec::single_parent(
            EntryTypeModality::Any,
            EntryTypeModality::Specific(EntryType::Reference),
        );
        let collection_d = EntryTypeSpec::single_parent(
            EntryTypeModality::Specific(EntryType::Article),
            EntryTypeModality::Specific(EntryType::Proceedings),
        );
        let tv_series = EntryTypeSpec::single_parent(
            EntryTypeModality::Specific(EntryType::Video),
            EntryTypeModality::Specific(EntryType::Video),
        );
        let thesis = EntryTypeSpec::with_single(EntryType::Thesis);
        let manuscript = EntryTypeSpec::with_single(EntryType::Manuscript);
        let art_container = EntryTypeSpec::single_parent(
            EntryTypeModality::Any,
            EntryTypeModality::Specific(EntryType::Artwork),
        );
        let art = EntryTypeSpec::new(
            EntryTypeModality::Alternate(vec![EntryType::Artwork, EntryType::Exhibition]),
            vec![],
        );
        let web_standalone = EntryTypeSpec::with_single(EntryType::WebItem);
        let web_contained_a = EntryTypeSpec::single_parent(
            EntryTypeModality::Any,
            EntryTypeModality::Alternate(vec![
                EntryType::Misc,
                EntryType::Blog,
                EntryType::WebItem,
            ]),
        );
        let web_contained_b = EntryTypeSpec::single_parent(
            EntryTypeModality::Specific(EntryType::WebItem),
            EntryTypeModality::Any,
        );
        let talk = EntryTypeSpec::single_parent(
            EntryTypeModality::Any,
            EntryTypeModality::Specific(EntryType::Conference),
        );

        if let Ok(i) = entry.check_with_spec(periodical) {
            return Self::PeriodicalItem(&entry.get_parents_ref().unwrap()[i[0]]);
        }
        if let Ok(i) = entry
            .check_with_spec(collection_a)
            .or_else(|_| entry.check_with_spec(collection_b))
            .or_else(|_| entry.check_with_spec(collection_c))
            .or_else(|_| entry.check_with_spec(collection_d))
        {
            return Self::CollectionItem(&entry.get_parents_ref().unwrap()[i[0]]);
        }
        if let Ok(i) = entry.check_with_spec(tv_series) {
            if entry.get_issue().is_ok() && entry.get_volume().is_ok() {
                return Self::TvSeries(&entry.get_parents_ref().unwrap()[i[0]]);
            }
        }
        if entry.check_with_spec(thesis).is_ok() {
            return Self::Thesis;
        }
        if entry.check_with_spec(manuscript).is_ok() {
            return Self::Manuscript;
        }
        if let Ok(i) = entry.check_with_spec(art_container) {
            return Self::ArtContainer(&entry.get_parents_ref().unwrap()[i[0]]);
        }
        if entry.check_with_spec(art).is_ok() {
            return Self::StandaloneArt;
        }
        if entry.check_with_spec(web_standalone).is_ok() {
            return Self::StandaloneWebItem;
        }
        if let Ok(i) = entry
            .check_with_spec(web_contained_a)
            .or_else(|_| entry.check_with_spec(web_contained_b))
        {
            return Self::WebItem(&entry.get_parents_ref().unwrap()[i[0]]);
        }
        if let Ok(i) = entry.check_with_spec(talk) {
            return Self::ConferenceTalk(&entry.get_parents_ref().unwrap()[i[0]]);
        }

        return Self::Generic;
    }
}

fn format_range<T: std::fmt::Display + PartialEq>(
    prefix_s: &str,
    prefix_m: &str,
    range: &Range<T>,
) -> String {
    let space = if prefix_s.is_empty() { "" } else { " " };
    if range.start == range.end {
        format!("{}{}{}", prefix_s, space, range.start)
    } else {
        format!("{}{}{}–{}", prefix_m, space, range.start, range.end)
    }
}

fn name_list(persons: &[Person]) -> Vec<String> {
    let mut names = vec![];

    for author in persons.iter() {
        let mut single = if let Some(prefix) = &author.prefix {
            format!("{} {}", prefix, author.name)
        } else {
            author.name.clone()
        };

        if let Some(initials) = author.get_initials(Some(".")) {
            single += ", ";
            single += &initials;
        }

        if let Some(suffix) = &author.suffix {
            single += ", ";
            single += suffix;
        }

        names.push(single);
    }

    names
}

fn ampersand_list(names: Vec<String>) -> String {
    let name_len = names.len() as i64;
    let mut res = String::new();

    for (index, name) in names.into_iter().enumerate() {
        if index > 19 && name_len > 20 && (index as i64) != name_len - 1 {
            // Element 20 or longer if longer than twenty and not last
            continue;
        }

        if index == 19 && name_len > 20 {
            res += "... ";
        } else {
            res += &name;
        }

        if (index as i64) <= name_len - 2 {
            res += ", ";
        }
        if (index as i64) == name_len - 2 {
            res += "& ";
        }
    }

    res
}

fn ed_vol_str(entry: &Entry) -> String {
    let vstr = if let Ok(vols) = entry.get_volume() {
        Some(format_range("Vol.", "Vols.", &vols))
    } else {
        None
    };

    let estr = if let Ok(ed) = entry.get_edition() {
        Some(match ed {
            NumOrStr::Number(e) => get_ordinal(e),
            NumOrStr::Str(s) => s,
        })
    } else {
        None
    };

    match (estr, vstr) {
        (None, None) => String::new(),
        (Some(e), None) => format!(" ({} ed.).", e),
        (None, Some(v)) => format!(" ({}).", v),
        (Some(e), Some(v)) => format!(" ({} ed., {}).", e, v),
    }
}

impl<'s> ApaBibliographyGenerator<'s> {
    pub fn new(entries: &'s [Entry]) -> Self {
        Self {
            entries,
            formatter: SentenceCaseTransformer::default(),
        }
    }

    fn get_author(&self, index: usize) -> String {
        let entry = &self.entries[index];
        ampersand_list(name_list(&entry.get_authors()))
    }

    fn get_date(&self, index: usize) -> String {
        let date = &self.entries[index].get_date();

        if let Ok(date) = date {
            match (date.month, date.day) {
                (None, _) => format!("({:04})", date.year),
                (Some(month), None) => {
                    format!("({:04}, {})", date.year, get_month_name(month).unwrap())
                }
                (Some(month), Some(day)) => format!(
                    "({:04}, {} {})",
                    date.year,
                    get_month_name(month).unwrap(),
                    day,
                ),
            }
        } else {
            "(n. d.)".to_string()
        }
    }

    fn get_retreival_date(&self, index: usize) -> Option<String> {
        let url = &self.entries[index].get_url();

        if let Ok(qurl) = url {
            let uv = qurl.value.as_str();
            let res = if let Some(date) = &qurl.visit_date {
                match (date.month, date.day) {
                    (None, _) => format!("Retrieved {:04}, from {}", date.year, uv),
                    (Some(month), None) => format!(
                        "Retrieved {} {:04}, from {}",
                        get_month_name(month).unwrap(),
                        date.year,
                        uv,
                    ),
                    (Some(month), Some(day)) => format!(
                        "(Retrieved {} {}, {:04}, from {})",
                        get_month_name(month).unwrap(),
                        day,
                        date.year,
                        uv,
                    ),
                }
            } else {
                uv.to_string()
            };

            Some(res)
        } else {
            None
        }
    }

    fn get_title(&self, index: usize) -> Option<String> {
        let entry = &self.entries[index];
        if let Ok(title) = entry.get_title_fmt(None, Some(&self.formatter)) {
            let multivol_spec = EntryTypeSpec::with_parents(EntryType::Book, vec![
                EntryTypeSpec::with_single(EntryType::Book),
            ]);

            let mut multivolume_parent = None;
            if let Ok(parents) = entry.check_with_spec(multivol_spec) {
                if entry.get_volume().is_ok() {
                    multivolume_parent = Some(parents[0]);
                }
            }

            let book_spec = EntryTypeSpec::new(
                EntryTypeModality::Alternate(vec![
                    EntryType::Book,
                    EntryType::Report,
                    EntryType::Reference,
                    EntryType::Anthology,
                    EntryType::Proceedings,
                ]),
                vec![],
            );

            let mut res = title.sentence_case;

            if let Some(mv_parent) = multivolume_parent {
                let p: &Entry = &entry.get_parents().unwrap()[mv_parent];
                let vols = entry.get_volume().unwrap();
                res = format!(
                    "{}: {} {}.",
                    p.get_title_fmt(None, Some(&self.formatter)).unwrap().sentence_case,
                    format_range("Vol.", "Vols.", &vols),
                    res
                );
            } else if (entry.get_volume().is_ok() || entry.get_edition().is_ok())
                && entry.check_with_spec(book_spec).is_ok()
            {
                res += &ed_vol_str(entry);
            } else {
                let lc = res.chars().last().unwrap_or('a');

                if lc != '?' && lc != '.' && lc != '!' {
                    res.push('.');
                }
            }

            Some(res)
        } else {
            None
        }
    }

    fn get_source(&self, index: usize) -> String {
        let entry = &self.entries[index];
        let st = SourceType::for_entry(&entry);
        let mut res = String::new();

        match st {
            SourceType::PeriodicalItem(parent) => {
                let mut comma = if let Ok(title) =
                    parent.get_title_fmt(None, Some(&self.formatter))
                {
                    res += &title.sentence_case;
                    true
                } else {
                    false
                };

                if parent.get_volume().is_ok() || parent.get_issue().is_ok() {
                    if comma {
                        res += ", ";
                    }

                    if let Ok(volume) = parent.get_volume() {
                        res += &format_range("", "", &volume);
                    }

                    if let Ok(issue) = parent.get_issue() {
                        res += &format!("({})", issue);
                    }
                    comma = true;
                }

                if entry.get_serial_number().is_ok() || entry.get_page_range().is_ok() {
                    if comma {
                        res += ", ";
                    }

                    if let Ok(sn) = entry.get_serial_number() {
                        res += &sn;
                    } else if let Ok(pages) = entry.get_page_range() {
                        res += &format_range("", "", &pages);
                    }

                    comma = true;
                }

                if comma {
                    res.push('.');
                }
            }
            SourceType::CollectionItem(parent) => {
                let mut comma = if let Ok(eds) = parent.get_editors() {
                    let names = name_list(&eds);
                    match names.len() {
                        0 => false,
                        1 => {
                            res += &format!("{} (Ed.)", names[0]);
                            true
                        }
                        _ => {
                            res += &format!("{} (Eds.)", ampersand_list(names));
                            true
                        }
                    }
                } else {
                    false
                };

                if let Ok(title) = parent.get_title_fmt(None, Some(&self.formatter)) {
                    if comma {
                        res += ", ";
                    }

                    res += &title.sentence_case;
                    comma = false;

                    if parent.get_volume().is_ok() || parent.get_edition().is_ok() {
                        res.push(' ');
                        res += &ed_vol_str(entry);
                    } else {
                        let lc = res.chars().last().unwrap_or('a');

                        if lc != '?' && lc != '.' && lc != '!' {
                            res.push('.');
                        }
                    }
                }

                if comma {
                    res += ".";
                }

                if !res.is_empty() {
                    res = format!("In {}", res);
                }

                if parent.get_publisher().is_ok() || parent.get_organization().is_ok() {
                    res.push(' ');

                    if let Ok(publisher) = parent.get_publisher() {
                        res += &publisher.value;
                    } else if let Ok(organization) = parent.get_organization() {
                        res += &organization;
                    }

                    res.push('.');
                }
            }
            SourceType::TvSeries(parent) => {
                let prods = parent
                    .get_affiliated_persons()
                    .map(|p| {
                        p.into_iter()
                            .filter(|x| x.1 == PersonRole::ExecutiveProducer)
                            .map(|x| x.0)
                            .flatten()
                            .collect::<Vec<Person>>()
                    })
                    .unwrap_or_else(|_| entry.get_authors());
                let mut comma = if !prods.is_empty() {
                    let names = name_list(&prods);
                    match names.len() {
                        0 => false,
                        1 => {
                            res += &format!("{} (Ed.)", names[0]);
                            true
                        }
                        _ => {
                            res += &format!("{} (Eds.)", ampersand_list(names));
                            true
                        }
                    }
                } else {
                    false
                };

                if let Ok(title) = parent.get_title_fmt(None, Some(&self.formatter)) {
                    if comma {
                        res += ", ";
                    }

                    res += &title.sentence_case;
                    comma = false;

                    if parent.get_volume().is_ok() || parent.get_edition().is_ok() {
                        res.push(' ');
                        res += &ed_vol_str(entry);
                    } else {
                        let lc = res.chars().last().unwrap_or('a');

                        if lc != '?' && lc != '.' && lc != '!' {
                            res.push('.');
                        }
                    }
                }

                if comma {
                    res += ".";
                }

                if !res.is_empty() {
                    res = format!("In {}", res);
                }

                if parent.get_publisher().is_ok() || parent.get_organization().is_ok() {
                    res.push(' ');

                    if let Ok(publisher) = parent.get_publisher() {
                        res += &publisher.value;
                    } else if let Ok(organization) = parent.get_organization() {
                        res += &organization;
                    }

                    res.push('.');
                }
            }
            SourceType::Thesis => {
                if let Ok(archive) = entry.get_archive() {
                    res += &archive.value;
                    res.push('.');
                } else if let Ok(org) = entry.get_organization() {
                    res += &org;
                    res.push('.');
                }
            }
            SourceType::Manuscript => {
                if let Ok(archive) = entry.get_archive() {
                    res += &archive.value;
                    res.push('.');
                }
            }
            SourceType::ArtContainer(parent) => {}
            SourceType::StandaloneArt => {}
            SourceType::StandaloneWebItem => {}
            SourceType::WebItem(parent) => {}
            SourceType::NewsItem(parent) => {}
            SourceType::ConferenceTalk(parent) => {}
            SourceType::Generic => {
                if entry.get_publisher().is_ok() || entry.get_organization().is_ok() {
                    if let Ok(publisher) = entry.get_publisher() {
                        res += &publisher.value;
                    } else if let Ok(organization) = entry.get_organization() {
                        res += &organization;
                    }

                    res.push('.');
                }
            }
        }

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::ApaBibliographyGenerator;
    use crate::types::EntryType;
    use crate::types::Person;
    use crate::Entry;

    #[test]
    fn name_list() {
        let p = vec![
            Person::from_strings(&vec!["van de Graf", "Judith"]),
            Person::from_strings(&vec!["Günther", "Hans-Joseph"]),
            Person::from_strings(&vec!["Mädje", "Laurenz Elias"]),
        ]
        .into_iter()
        .map(|e| e.unwrap())
        .collect();
        let mut entry = Entry::new("test", EntryType::NewspaperIssue);
        entry.set_authors(p);
        let ets = vec![entry];

        let apa = ApaBibliographyGenerator::new(&ets);
        assert_eq!(
            "van de Graf, J., Günther, H.-J., & Mädje, L. E.",
            apa.get_author(0)
        );
    }
}

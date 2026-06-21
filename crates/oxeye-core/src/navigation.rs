//! Structured-navigation primitives: classifying UI elements into navigation categories and
//! summarising a window/application's structure.
//!
//! This is the platform-agnostic **functional core** of "browse-mode" navigation — pure and
//! deterministic, unit-tested without any accessibility back-end. A platform crate reads roles
//! from the accessibility tree, calls [`classify`], and feeds the results to [`summarize`]
//! (and, later, to by-type next/previous movement built on the same categories).

/// A structural category a UI element can be navigated or summarised by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NavCategory {
    /// A section heading.
    Heading,
    /// A navigational landmark / region.
    Landmark,
    /// A hyperlink.
    Link,
    /// A push/toggle button.
    Button,
    /// An interactive form control (entry, checkbox, combo box, slider, …).
    FormField,
    /// A list.
    List,
    /// A table or grid.
    Table,
    /// An image or icon.
    Image,
}

impl NavCategory {
    /// Categories in the order a structure summary reports them.
    const ORDER: [NavCategory; 8] = [
        NavCategory::Heading,
        NavCategory::Landmark,
        NavCategory::Link,
        NavCategory::Button,
        NavCategory::FormField,
        NavCategory::List,
        NavCategory::Table,
        NavCategory::Image,
    ];

    /// The singular spoken label (e.g. "button").
    #[must_use]
    pub fn singular(self) -> &'static str {
        match self {
            NavCategory::Heading => "heading",
            NavCategory::Landmark => "landmark",
            NavCategory::Link => "link",
            NavCategory::Button => "button",
            NavCategory::FormField => "form field",
            NavCategory::List => "list",
            NavCategory::Table => "table",
            NavCategory::Image => "image",
        }
    }

    /// The plural spoken label (e.g. "buttons").
    #[must_use]
    pub fn plural(self) -> &'static str {
        match self {
            NavCategory::Heading => "headings",
            NavCategory::Landmark => "landmarks",
            NavCategory::Link => "links",
            NavCategory::Button => "buttons",
            NavCategory::FormField => "form fields",
            NavCategory::List => "lists",
            NavCategory::Table => "tables",
            NavCategory::Image => "images",
        }
    }
}

/// Classify an (AT-SPI) role name into a navigation category, if it is one oxeye surfaces.
/// Role names follow AT-SPI's `Role::name()` (e.g. `"push button"`, `"heading"`, `"link"`).
#[must_use]
pub fn classify(role: &str) -> Option<NavCategory> {
    use NavCategory::{Button, FormField, Heading, Image, Landmark, Link, List, Table};
    let category = match role {
        "heading" => Heading,
        "landmark" => Landmark,
        "link" => Link,
        "push button" | "toggle button" => Button,
        "entry" | "text" | "password text" | "spin button" | "combo box" | "check box"
        | "radio button" | "slider" => FormField,
        "list" | "list box" => List,
        "table" | "tree table" | "tree" => Table,
        "image" | "icon" => Image,
        _ => return None,
    };
    Some(category)
}

/// Summarise a window/application's structure from its elements' categories, as a spoken phrase
/// in a fixed order with pluralisation, e.g. `"3 headings, 12 buttons, 4 links"`.
///
/// Returns `None` when nothing notable is present.
#[must_use]
pub fn summarize<I>(categories: I) -> Option<String>
where
    I: IntoIterator<Item = Option<NavCategory>>,
{
    let present: Vec<NavCategory> = categories.into_iter().flatten().collect();
    let parts: Vec<String> = NavCategory::ORDER
        .iter()
        .filter_map(|&category| {
            let count = present.iter().filter(|&&c| c == category).count();
            (count > 0).then(|| {
                let label = if count == 1 {
                    category.singular()
                } else {
                    category.plural()
                };
                format!("{count} {label}")
            })
        })
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::{classify, summarize, NavCategory};

    #[test]
    fn classifies_known_roles_and_ignores_others() {
        assert_eq!(classify("heading"), Some(NavCategory::Heading));
        assert_eq!(classify("push button"), Some(NavCategory::Button));
        assert_eq!(classify("entry"), Some(NavCategory::FormField));
        assert_eq!(classify("check box"), Some(NavCategory::FormField));
        assert_eq!(classify("link"), Some(NavCategory::Link));
        assert_eq!(classify("filler"), None);
        assert_eq!(classify("panel"), None);
    }

    #[test]
    fn summarize_counts_pluralizes_and_orders() {
        let cats = vec![
            Some(NavCategory::Button),
            Some(NavCategory::Heading),
            None,
            Some(NavCategory::Button),
            Some(NavCategory::Link),
        ];
        // Reported in ORDER (heading, link, button), pluralised by count.
        assert_eq!(
            summarize(cats).as_deref(),
            Some("1 heading, 1 link, 2 buttons")
        );
    }

    #[test]
    fn summarize_is_none_when_nothing_notable() {
        assert_eq!(summarize(vec![None, None]), None);
        assert_eq!(summarize(Vec::<Option<NavCategory>>::new()), None);
    }
}

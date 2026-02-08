use maud::{Markup, html};

pub struct Breadcrumb<'a> {
    pub label: &'a str,
    pub href: Option<&'a str>,
}

pub fn breadcrumbs(items: &[Breadcrumb]) -> Markup {
    html! {
        nav.breadcrumbs {
            ol.breadcrumbs-list {
                @for item in items {
                    li.breadcrumb {
                        @if let Some(href) = item.href {
                            a href=(href) { (item.label) }
                        } @else {
                            (item.label)
                        }
                    }
                }
            }
        }
    }
}

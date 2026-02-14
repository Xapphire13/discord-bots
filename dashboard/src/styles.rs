mod global {
    turf::style_sheet!("assets/global.css");
}

mod breadcrumbs {
    turf::style_sheet!("assets/breadcrumbs.css");
}

mod bot_list {
    turf::style_sheet!("assets/bot_list.css");
}

pub use bot_list::ClassName as BotList;
pub use breadcrumbs::ClassName as Breadcrumbs;

use std::sync::LazyLock;

pub static ALL: LazyLock<String> = LazyLock::new(|| {
    [
        global::STYLE_SHEET,
        breadcrumbs::STYLE_SHEET,
        bot_list::STYLE_SHEET,
    ]
    .join("\n")
});

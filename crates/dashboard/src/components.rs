use maud::{DOCTYPE, Markup, PreEscaped, html};

pub fn layout(title: &str, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                // Tailwind via CDN for now (no local build pipeline needed)
                script src="https://cdn.tailwindcss.com" {};
                // HTMX via CDN for now (no local build pipeline needed)
                script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.6/dist/htmx.min.js" integrity="sha384-Akqfrbj/HpNVo8k11SXBb6TlBWmXXlYQrCSqEWmyKJe+hDm3Z/B2WVG4smwBkRVm" crossorigin="anonymous" {};
                // Prefer dark background and system dark mode
                script { (PreEscaped(r#"
                  try {
                    tailwind.config = { darkMode: 'media' };
                  } catch (_) {}
                "#)) }
            }
            body class="bg-slate-950 text-slate-100 min-h-screen" { (body) }
        }
    }
}

pub fn topbar() -> Markup {
    html! {
        header class="w-full border-b border-slate-800 bg-slate-900 text-slate-100 px-4 py-3 flex items-center gap-3" {
            strong class="text-base" { "Adminboard" }
            button id="tab-sources" class="tab px-3 py-1.5 rounded-md border border-slate-700 bg-slate-700/20 hover:bg-slate-700/40 transition-colors" { "Sources" }
            button id="tab-added" class="tab px-3 py-1.5 rounded-md border border-slate-700 bg-transparent hover:bg-slate-700/40 transition-colors" { "Added" }
            span class="flex-1" {}
            button id="signin" class="px-3 py-1.5 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors" { "Sign in" }
            button id="signout" class="px-3 py-1.5 rounded-md border border-slate-700 bg-slate-800 hover:bg-slate-700 transition-colors hidden" { "Sign out" }
        }
    }
}

pub fn page_container(content: Markup) -> Markup {
    html! {
        main class="w-full mx-auto p-4" { (content) }
    }
}

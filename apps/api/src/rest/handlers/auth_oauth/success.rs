use axum::{extract::Query, response::Html};
use maud::{DOCTYPE, PreEscaped, html};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SuccessQuery {
    pub token: Option<String>,
}

pub async fn auth_success(Query(q): Query<SuccessQuery>) -> Html<String> {
    let token = q.token.unwrap_or_default();

    let page = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Authentication Success" }
                style {
                    "body{font-family:ui-sans-serif,system-ui,-apple-system,Segoe UI,Roboto,Ubuntu,Cantarell,Noto Sans,'Helvetica Neue',Arial,'Apple Color Emoji','Segoe UI Emoji';padding:2rem;background:#0b1020;color:#e6edf3}"
                    "main{max-width:720px;margin:0 auto}"
                    ".card{background:#0f152b;border:1px solid #26304b;border-radius:12px;padding:20px}"
                    "h1{font-size:1.25rem;margin:0 0 1rem}"
                    ".row{display:flex;gap:.5rem;align-items:center}"
                    "input{flex:1;padding:.6rem .8rem;border:1px solid #2b3754;border-radius:8px;background:#0b1020;color:#e6edf3}"
                    "button{padding:.6rem .9rem;border:1px solid #2b3754;border-radius:8px;background:#1a2442;color:#e6edf3;cursor:pointer}"
                    "button:hover{background:#223056}"
                    ".note{margin-top:.75rem;color:#9fb1d1;font-size:.9rem}"
                    ".warn{color:#f3d28f}"
                }
            }
            body {
                main {
                    div class="card" {
                        h1 { "Authentication successful" }
                        p class="note" { "This page shows your JWT for development purposes. " span class="warn" { "Do not share it." } }
                        div class="row" {
                            input id="jwt" type="text" readonly placeholder="No token provided" value=(token) {}
                            button id="copy" type="button" onclick="copyJwt()" { "Copy" }
                        }
                        p id="copied" class="note" style="display:none" { "Copied to clipboard." }
                    }
                }
                script { (PreEscaped(r#"
                    function copyJwt(){
                        var el = document.getElementById('jwt');
                        if(!el || !el.value){ return; }
                        var show = function(){
                            var m = document.getElementById('copied');
                            if(m){
                                m.style.display='block';
                                setTimeout(function(){ m.style.display='none'; }, 1500);
                            }
                        };
                        if (navigator.clipboard && window.isSecureContext) {
                            navigator.clipboard.writeText(el.value).then(show).catch(fallback);
                        } else {
                            fallback();
                        }
                        function fallback(){
                            var prev = el.getAttribute('readonly');
                            if(prev !== null){ el.removeAttribute('readonly'); }
                            el.select(); el.setSelectionRange(0, el.value.length);
                            try { document.execCommand('copy'); show(); } catch(e){}
                            if(prev !== null){ el.setAttribute('readonly',''); }
                            if(window.getSelection){ window.getSelection().removeAllRanges(); }
                        }
                    }
                "#)) }
            }
        }
    };

    Html(page.into_string())
}

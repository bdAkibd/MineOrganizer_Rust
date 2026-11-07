use tauri::command;

#[command]
pub fn get_user_manual_cmd(locale: Option<String>) -> String {
    let loc = locale.as_deref().unwrap_or("en");
    match loc {
        "ru" => include_str!("../../../docs/manual_ru.html"),
        "de" => include_str!("../../../docs/manual_de.html"),
        "es" => include_str!("../../../docs/manual_es.html"),
        "fr" => include_str!("../../../docs/manual_fr.html"),
        "uk" => include_str!("../../../docs/manual_uk.html"),
        "it" => include_str!("../../../docs/manual_it.html"),
        "zh_CN" => include_str!("../../../docs/manual_zh_CN.html"),
        "zh_TW" => include_str!("../../../docs/manual_zh_TW.html"),
        "ja" => include_str!("../../../docs/manual_ja.html"),
        "pt" => include_str!("../../../docs/manual_pt.html"),
        _ => include_str!("../../../docs/manual_en.html"),
    }
    .to_string()
}

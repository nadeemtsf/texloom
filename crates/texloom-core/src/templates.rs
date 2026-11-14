//! Bundled, editable starting projects. No user data or hidden dependencies.
pub struct Template {
    pub title: &'static str,
    pub source: &'static str,
    pub engine: &'static str,
}
pub fn get(id: &str) -> Option<Template> {
    let (title, source, engine) = match id {
        "article" => (
            "Article",
            include_str!("../../../resources/templates/article.tex"),
            "pdflatex",
        ),
        "letter" => (
            "Letter",
            include_str!("../../../resources/templates/letter.tex"),
            "pdflatex",
        ),
        "cv" => (
            "CV",
            include_str!("../../../resources/templates/cv.tex"),
            "pdflatex",
        ),
        "multilingual" => (
            "English, Arabic and Hebrew",
            include_str!("../../../resources/templates/multilingual.tex"),
            "xelatex",
        ),
        _ => return None,
    };
    Some(Template {
        title,
        source,
        engine,
    })
}

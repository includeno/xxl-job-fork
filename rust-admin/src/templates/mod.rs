use askama::Template;

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub app_name: &'a str,
    pub tagline: &'a str,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate<'a> {
    pub app_name: &'a str,
    pub summary_endpoint: &'a str,
    pub chart_endpoint: &'a str,
}

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
    pub active_nav: &'a str,
    pub summary_endpoint: &'a str,
    pub chart_endpoint: &'a str,
}

#[derive(Template)]
#[template(path = "job-groups.html")]
pub struct JobGroupsTemplate<'a> {
    pub app_name: &'a str,
    pub active_nav: &'a str,
    pub job_groups_endpoint: &'a str,
}

#[derive(Template)]
#[template(path = "job-info.html")]
pub struct JobInfoTemplate<'a> {
    pub app_name: &'a str,
    pub active_nav: &'a str,
    pub job_groups_endpoint: &'a str,
    pub job_info_endpoint: &'a str,
    pub job_info_next_trigger_endpoint: &'a str,
}

#[derive(Template)]
#[template(path = "job-logs.html")]
pub struct JobLogsTemplate<'a> {
    pub app_name: &'a str,
    pub active_nav: &'a str,
    pub job_groups_endpoint: &'a str,
    pub job_logs_endpoint: &'a str,
}

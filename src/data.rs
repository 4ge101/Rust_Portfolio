use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::opener::is_allowed_url;

const BUNDLED_PROFILE: &str = include_str!("../data/profile.toml");
const BUNDLED_PROJECTS: &str = include_str!("../data/projects.toml");
const BUNDLED_SKILLS: &str = include_str!("../data/skills.toml");
const BUNDLED_EXPERIENCE: &str = include_str!("../data/experience.toml");
const BUNDLED_ACHIEVEMENTS: &str = include_str!("../data/achievements.toml");

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub title: String,
    pub tagline: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    #[serde(default)]
    pub bio: Vec<String>,
    #[serde(default)]
    pub focus: Vec<String>,
    #[serde(default)]
    pub interests: Vec<String>,
    pub philosophy: Option<String>,
    #[serde(default)]
    pub portrait: PortraitConfig,
    pub github: Option<Github>,
    #[serde(default)]
    pub contact: Vec<ContactLink>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PortraitConfig {
    pub width: u16,
    pub invert: bool,
}

impl Default for PortraitConfig {
    fn default() -> Self {
        Self {
            width: 56,
            invert: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Github {
    pub username: String,
    pub profile_url: Option<String>,
    #[serde(default)]
    pub repos: Vec<Repo>,
}

impl Github {
    pub fn profile_link(&self) -> String {
        self.profile_url
            .clone()
            .unwrap_or_else(|| format!("https://github.com/{}", self.username))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repo {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
}

impl Repo {
    pub fn link(&self, github: &Github) -> String {
        self.url
            .clone()
            .unwrap_or_else(|| format!("https://github.com/{}/{}", github.username, self.name))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactLink {
    pub label: String,
    pub value: String,
    pub url: Option<String>,
}

impl ContactLink {
    pub fn target(&self) -> Option<String> {
        if let Some(url) = &self.url {
            return Some(url.clone());
        }
        let value = self.value.trim();
        if value.starts_with("http://") || value.starts_with("https://") {
            return Some(value.to_owned());
        }
        let looks_like_email = value.contains('@')
            && !value.contains(char::is_whitespace)
            && !value.contains("://")
            && !value.starts_with('@')
            && !value.ends_with('@');
        looks_like_email.then(|| format!("mailto:{value}"))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub technologies: Vec<String>,
    pub status: Option<String>,
    pub github: Option<String>,
    pub website: Option<String>,
    #[serde(default)]
    pub featured: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillCategory {
    pub name: String,
    #[serde(default)]
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperienceEntry {
    pub period: String,
    pub title: String,
    pub organization: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Achievement {
    pub title: String,
    pub category: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectsFile {
    #[serde(default)]
    projects: Vec<Project>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillsFile {
    #[serde(default)]
    categories: Vec<SkillCategory>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperienceFile {
    #[serde(default)]
    experience: Vec<ExperienceEntry>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct AchievementsFile {
    #[serde(default)]
    achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Default)]
pub struct Portfolio {
    pub profile: Profile,
    pub projects: Vec<Project>,
    pub skills: Vec<SkillCategory>,
    pub experience: Vec<ExperienceEntry>,
    pub achievements: Vec<Achievement>,
}

#[derive(Debug)]
pub struct Loaded {
    pub portfolio: Portfolio,
    pub warnings: Vec<String>,
}

pub fn load(dir: Option<&Path>) -> Loaded {
    let mut warnings = Vec::new();

    let profile: Profile = load_file(dir, "profile.toml", BUNDLED_PROFILE, &mut warnings);
    let projects: ProjectsFile = load_file(dir, "projects.toml", BUNDLED_PROJECTS, &mut warnings);
    let skills: SkillsFile = load_file(dir, "skills.toml", BUNDLED_SKILLS, &mut warnings);
    let experience: ExperienceFile =
        load_file(dir, "experience.toml", BUNDLED_EXPERIENCE, &mut warnings);
    let achievements: AchievementsFile = load_file(
        dir,
        "achievements.toml",
        BUNDLED_ACHIEVEMENTS,
        &mut warnings,
    );

    let portfolio = Portfolio {
        profile,
        projects: projects.projects,
        skills: skills.categories,
        experience: experience.experience,
        achievements: achievements.achievements,
    };
    warnings.extend(portfolio.validate());

    Loaded {
        portfolio,
        warnings,
    }
}

fn load_file<T: DeserializeOwned + Default>(
    dir: Option<&Path>,
    name: &str,
    bundled: &str,
    warnings: &mut Vec<String>,
) -> T {
    if let Some(dir) = dir {
        let path = dir.join(name);
        match fs::read_to_string(&path) {
            Ok(text) => match toml::from_str(&text) {
                Ok(value) => return value,
                Err(err) => warnings.push(format!(
                    "{name}: {} (using bundled copy)",
                    one_line(&err.to_string())
                )),
            },
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => warnings.push(format!("{name}: {err} (using bundled copy)")),
        }
    }
    match toml::from_str(bundled) {
        Ok(value) => value,
        Err(err) => {
            warnings.push(format!(
                "bundled {name} is invalid: {}",
                one_line(&err.to_string())
            ));
            T::default()
        }
    }
}

fn one_line(message: &str) -> String {
    let mut parts = message.lines().filter(|line| {
        let line = line.trim();
        !line.is_empty() && !line.starts_with('|') && !line.contains(" | ")
    });
    let first = parts.next().unwrap_or("parse error").trim().to_owned();
    match parts.next_back() {
        Some(last) => format!("{first} - {}", last.trim()),
        None => first,
    }
}

pub fn default_config_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    let dir = base.join("alixsami");
    dir.is_dir().then_some(dir)
}

impl Portfolio {
    pub fn validate(&self) -> Vec<String> {
        let mut problems = Vec::new();
        let profile = &self.profile;

        if profile.name.trim().is_empty() {
            problems.push("profile.toml: `name` is empty".to_owned());
        }
        if profile.title.trim().is_empty() {
            problems.push("profile.toml: `title` is empty".to_owned());
        }
        if !(10..=200).contains(&profile.portrait.width) {
            problems.push(format!(
                "profile.toml: portrait.width {} is outside 10..=200",
                profile.portrait.width
            ));
        }

        if let Some(github) = &profile.github {
            let valid_user = !github.username.is_empty()
                && github
                    .username
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !valid_user {
                problems.push(format!(
                    "profile.toml: github.username {:?} is not a valid GitHub username",
                    github.username
                ));
            }
            check_url(
                &mut problems,
                "github.profile_url",
                github.profile_url.as_deref(),
            );
            for repo in &github.repos {
                check_url(
                    &mut problems,
                    &format!("github.repos[{}].url", repo.name),
                    repo.url.as_deref(),
                );
            }
        }

        for link in &profile.contact {
            check_url(
                &mut problems,
                &format!("contact[{}].url", link.label),
                link.url.as_deref(),
            );
        }

        for project in &self.projects {
            if project.name.trim().is_empty() {
                problems.push("projects.toml: a project has an empty `name`".to_owned());
            }
            check_url(
                &mut problems,
                &format!("projects[{}].github", project.name),
                project.github.as_deref(),
            );
            check_url(
                &mut problems,
                &format!("projects[{}].website", project.name),
                project.website.as_deref(),
            );
        }

        for achievement in &self.achievements {
            check_url(
                &mut problems,
                &format!("achievements[{}].url", achievement.title),
                achievement.url.as_deref(),
            );
        }

        problems
    }
}

fn check_url(problems: &mut Vec<String>, field: &str, url: Option<&str>) {
    if let Some(url) = url {
        if !is_allowed_url(url) {
            problems.push(format!(
                "{field}: {url:?} must be a full http(s) or mailto: link"
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_data_parses_and_validates() {
        let loaded = load(None);
        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert!(!loaded.portfolio.profile.name.is_empty());
        assert!(!loaded.portfolio.projects.is_empty());
        assert!(!loaded.portfolio.skills.is_empty());
        assert!(!loaded.portfolio.experience.is_empty());
    }

    #[test]
    fn malformed_override_falls_back_with_a_warning() {
        let dir = std::env::temp_dir().join(format!("portfolio-test-bad-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("skills.toml"), "this is = = not toml").unwrap();
        fs::write(dir.join("projects.toml"), "[[projects]]\nnaem = \"typo\"\n").unwrap();

        let loaded = load(Some(&dir));
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(loaded.warnings.len(), 2, "{:?}", loaded.warnings);

        assert!(!loaded.portfolio.skills.is_empty());
        assert!(!loaded.portfolio.projects.is_empty());
    }

    #[test]
    fn override_replaces_bundled_file() {
        let dir = std::env::temp_dir().join(format!("portfolio-test-ok-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("projects.toml"),
            "[[projects]]\nname = \"Mine\"\ngithub = \"https://github.com/x/y\"\n",
        )
        .unwrap();

        let loaded = load(Some(&dir));
        fs::remove_dir_all(&dir).unwrap();

        assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
        assert_eq!(loaded.portfolio.projects.len(), 1);
        assert_eq!(loaded.portfolio.projects[0].name, "Mine");
    }

    #[test]
    fn validation_flags_bad_urls_and_names() {
        let mut portfolio = load(None).portfolio;
        portfolio.projects[0].github = Some("javascript:alert(1)".to_owned());
        portfolio.profile.name.clear();
        let problems = portfolio.validate();
        assert_eq!(problems.len(), 2, "{problems:?}");
    }

    #[test]
    fn contact_targets() {
        let link = |value: &str, url: Option<&str>| ContactLink {
            label: "x".to_owned(),
            value: value.to_owned(),
            url: url.map(str::to_owned),
        };
        assert_eq!(
            link("me@example.com", None).target().as_deref(),
            Some("mailto:me@example.com")
        );
        assert_eq!(
            link("https://example.com/me", None).target().as_deref(),
            Some("https://example.com/me")
        );
        assert_eq!(
            link("@handle", Some("https://x.com/handle"))
                .target()
                .as_deref(),
            Some("https://x.com/handle")
        );
        assert_eq!(link("@handle", None).target(), None);
        assert_eq!(link("call me maybe", None).target(), None);
    }

    #[test]
    fn github_links_are_derived() {
        let github = Github {
            username: "octocat".to_owned(),
            profile_url: None,
            repos: vec![],
        };
        let repo = Repo {
            name: "hello".to_owned(),
            description: None,
            url: None,
        };
        assert_eq!(github.profile_link(), "https://github.com/octocat");
        assert_eq!(repo.link(&github), "https://github.com/octocat/hello");
    }
}
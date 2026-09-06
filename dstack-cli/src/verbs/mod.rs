// verbs/mod.rs
// Every verb module of the roster and the two collectors the registry and the runner read.

use crate::core::verb::Verb;
use crate::selftest::Selftest;

pub mod ask;
pub mod decision;
pub mod doctor;
pub mod exec;
pub mod gate;
pub mod hook;
pub mod init;
pub mod issue;
pub mod ledger;
pub mod lint;
pub mod plan;
pub mod prompt;
pub mod quick;
pub mod report;
pub mod request;
pub mod review;
pub mod run;
pub mod run_view;
pub mod status;
pub mod verify;

/// Every handler of the roster, in the order the shell files were sourced. The registry attaches
/// them by name, so a handler that answers no roster entry is a test failure, not a hidden verb.
pub fn all_verbs() -> Vec<Box<dyn Verb>> {
    let mut verbs = Vec::new();
    verbs.extend(init::verbs());
    verbs.extend(run::verbs());
    verbs.extend(run_view::verbs());
    verbs.extend(status::verbs());
    verbs.extend(exec::verbs());
    verbs.extend(prompt::verbs());
    verbs.extend(request::verbs());
    verbs.extend(ask::verbs());
    verbs.extend(decision::verbs());
    verbs.extend(ledger::verbs());
    verbs.extend(plan::verbs());
    verbs.extend(review::verbs());
    verbs.extend(verify::verbs());
    verbs.extend(report::verbs());
    verbs.extend(quick::verbs());
    verbs.extend(issue::verbs());
    verbs.extend(gate::verbs());
    verbs.extend(lint::verbs());
    verbs.extend(hook::verbs());
    verbs.extend(doctor::verbs());
    verbs
}

/// Every checker the fixture runner (doctor --self) drives.
pub fn all_selftests() -> Vec<Box<dyn Selftest>> {
    let mut selftests = Vec::new();
    selftests.extend(init::selftests());
    selftests.extend(run::selftests());
    selftests.extend(run_view::selftests());
    selftests.extend(status::selftests());
    selftests.extend(exec::selftests());
    selftests.extend(request::selftests());
    selftests.extend(ask::selftests());
    selftests.extend(decision::selftests());
    selftests.extend(ledger::selftests());
    selftests.extend(plan::selftests());
    selftests.extend(review::selftests());
    selftests.extend(verify::selftests());
    selftests.extend(report::selftests());
    selftests.extend(quick::selftests());
    selftests.extend(issue::selftests());
    selftests.extend(gate::selftests());
    selftests.extend(lint::selftests());
    selftests.extend(hook::selftests());
    selftests.extend(doctor::selftests());
    selftests
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::registry::ROSTER;

    #[test]
    fn r13__every_handler_is_a_roster_entry() {
        let mut seen: Vec<&'static str> = Vec::new();
        for verb in all_verbs() {
            let name = verb.name();
            assert!(
                ROSTER.iter().any(|(entry, _)| *entry == name),
                "{name} is not on the roster"
            );
            assert!(!seen.contains(&name), "{name} has two handlers");
            seen.push(name);
        }
    }

    #[test]
    fn r05__every_selftest_has_a_fixture_directory() {
        let home = crate::core::roots::Home::resolve().expect("repository");
        for selftest in all_selftests() {
            let dir = home.home.join("lint/fixtures").join(selftest.checker());
            assert!(
                dir.is_dir(),
                "no fixture directory for {}",
                selftest.checker()
            );
        }
    }
}

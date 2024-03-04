use std::{collections::BTreeMap, fmt::Display, io::Write};

type ScriptResult = Result<(), Box<dyn ::std::fmt::Display>>;

pub struct Script {
    pub desc: &'static str,
    f: Box<dyn Fn() -> ScriptResult>,
}

pub struct ScriptManager<'a> {
    pub name: &'a str,
    scripts: BTreeMap<usize, Script>,
}

impl<'a> Display for ScriptManager<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = "========== ScriptManager ==========";
        let footer = "=".repeat(header.len());

        writeln!(f, "{}", header)?;
        for (idx, script) in &self.scripts {
            writeln!(f, "[{}] {}", idx, script.desc)?;
        }
        write!(f, "{}", footer)?;

        Ok(())
    }
}

impl<'a> ScriptManager<'a> {
    pub fn new(name: Option<&'a str>, scripts: BTreeMap<usize, Script>) -> Self {
        Self {
            name: match name {
                Some(n) => n,
                None => "ScriptManager",
            },
            scripts,
        }
    }

    pub fn run(&self) -> Result<(), String> {
        println!("{}\n", self);
        print!("Enter script ID: ");
        std::io::stdout()
            .flush()
            .expect("should be able to flush output buffer");

        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("can read input from user");

        let opt = buf.trim().parse::<usize>().expect("user gave valid input");
        (self.scripts[&opt].f)().map_err(|e| format!("{}", e))
    }

    #[cfg(test)]
    fn run_script(&self, idx: usize) -> Result<(), String> {
        (self.scripts[&idx].f)().map_err(|e| format!("{}", e))
    }
}

#[macro_export]
macro_rules! prsm {
    ($([$idx:literal] $desc:literal => $f:expr),*) => {
        ScriptManager::new(None, [$(
                (
                    $idx,
                    crate::Script {
                        desc: $desc,
                        f: Box::new(move || $f
                            .map_err(|e| Box::new(e) as Box<dyn ::std::fmt::Display>))
                    }
                )
            ),*]
            .into_iter()
            .collect::<BTreeMap<usize, crate::Script>>()
        )
    };

    ($manager_name:ident { $([$idx:literal] $desc:literal => $f:expr),* }) => {
        ScriptManager::new(Some(stringify!($manager_name)), [$(
                (
                    $idx,
                    crate::Script {
                        desc: $desc,
                        f: Box::new(move || $f
                            .map_err(|e| Box::new(e) as Box<dyn ::std::fmt::Display>))
                    }
                )
            ),*]
            .into_iter()
            .collect::<BTreeMap<usize, crate::Script>>()
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prsm_manager_default_name() {
        let x = || -> Result<(), usize> { Ok(()) };
        let sm = prsm! {
            [1] "Test x" => x()
        };

        assert_eq!(sm.name, "ScriptManager");
    }

    #[test]
    fn prsm_manager_name() {
        let x = || -> Result<(), usize> { Ok(()) };
        let sm = prsm! {
            TestManager {
                [1] "Test x" => x()
            }
        };

        assert_eq!(sm.name, "TestManager");
    }

    mod external_module {
        pub fn x() -> Result<(), usize> {
            Ok(())
        }

        pub fn y() -> Result<(), usize> {
            Ok(())
        }

        pub fn z(a: usize) -> Result<(), usize> {
            Err(a + 3)
        }
    }

    #[test]
    fn prsm_function_external() {
        let sm = prsm! {
            [1] "x" => external_module::x(),
            [2] "y" => external_module::y(),
            [3] "z" => external_module::z(0)
        };

        assert!(matches!(sm.run_script(1), Ok(())));
        assert!(matches!(sm.run_script(2), Ok(())));
        assert!(matches!(sm.run_script(3), Err(e) if e == "3"));
        assert_eq!(sm.scripts.len(), 3);
    }

    #[test]
    fn prsm_function_nested() {
        fn x() -> Result<(), usize> {
            Ok(())
        }

        fn y() -> Result<(), usize> {
            Ok(())
        }

        fn z(a: usize) -> Result<(), usize> {
            Err(a + 3)
        }

        let sm = prsm! {
            [1] "x" => x(),
            [2] "y" => y(),
            [3] "z" => z(0)
        };

        assert!(matches!(sm.run_script(1), Ok(())));
        assert!(matches!(sm.run_script(2), Ok(())));
        assert!(matches!(sm.run_script(3), Err(e) if e == "3"));
        assert_eq!(sm.scripts.len(), 3);
    }

    #[test]
    fn prsm_closure_no_capture() {
        let x = || -> Result<(), usize> { Ok(()) };
        let y = || -> Result<(), usize> { Ok(()) };
        let z = |a: usize| -> Result<(), usize> { Err(a + 3) };

        let sm = prsm! {
            [1] "x" => x(),
            [2] "y" => y(),
            [3] "z" => z(0)
        };

        assert!(matches!(sm.run_script(1), Ok(())));
        assert!(matches!(sm.run_script(2), Ok(())));
        assert!(matches!(sm.run_script(3), Err(e) if e == "3"));
        assert_eq!(sm.scripts.len(), 3);
    }

    #[test]
    fn prsm_closure_capture() {
        let x = || -> Result<(), usize> { Ok(()) };
        let y = || -> Result<(), usize> { Ok(()) };

        let b = 2;
        let z = move |a: usize| -> Result<(), usize> { Err(a + b) };

        let sm = prsm! {
            [1] "x" => x(),
            [2] "y" => y(),
            [3] "z" => z(5)
        };

        assert!(matches!(sm.run_script(1), Ok(())));
        assert!(matches!(sm.run_script(2), Ok(())));
        assert!(matches!(sm.run_script(3), Err(e) if e == "7"));
        assert_eq!(sm.scripts.len(), 3);
    }
}

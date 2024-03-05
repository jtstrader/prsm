#![deny(unsafe_code)]

//! ## PRSM - Project Script Manager
//! `prsm` (pronounced "prism") aims to speed up the process of writing simple project management
//! CLI applications. It's common to have a custom suite of formatting, linting, and debugging
//! scripts in separate shell/scripting files. However, for those interested in using Rust for
//! these purposes, it can be daunting to set up their scripts compared to others who use simpler
//! languages such as Python.
//!
//! The intent of `prsm` is to reduce any and all complexity of setting up the script manager so
//! you, the developer, can focus more time and energy into your management scripts. You're already
//! using Rust rather than the simpler alternatives. Why introduce *even more* complexity into your
//! life?
//!
//! Using `prsm` is easy thanks to the [`prsm`] macro.
//! ```rust,no_run
//! use prsm::prsm;
//!
//! fn format() -> Result<(), std::io::Error> { Ok(()) }
//! fn lint() -> Result<(), std::io::Error> { Ok(()) }
//!
//! let script_manager = prsm! {
//!     [1] "Format repository files" => format(),
//!     [2] "Lint Rust files" => lint()
//! };
//!
//! script_manager.run();
//! ```
//!
//! Note that `prsm` is a library dedicated to abstracting away the *setup process* of a script
//! manager. It is not interested in the explicit returns that you may have for your functions that
//! manage your project. It's best to use stateless functions for `prsm` that do not have
//! meaningful return values, as `prsm` will throw away any return value (other than errors, which
//! are returned for debugging purposes).

use std::{
    collections::BTreeMap,
    fmt::{Debug, Display},
    io::Write,
};

/// Import this to include all necessary `prsm` features to get your script manager up and running.
pub mod prelude {
    pub use crate::{prsm, PrsmDisplay, ScriptManager};
}

/// A displayable type that can be both displayed (for error handling) and debugged (for unwrapping).
pub trait PrsmDisplay: Display + Debug {}
impl<T> PrsmDisplay for T where T: Display + Debug {}

/// The required return type for `prsm` functions. If you wish to explicitly declare your return type
/// as one that does not fail, please use [`Infallible`](::std::convert::Infallible).
pub type ScriptResult = Result<(), Box<dyn PrsmDisplay>>;

/// A `prsm` script that can be called through the [`ScriptManager`].
///
/// These should almost never be manually constructed, and should instead be constructed through
/// the [`prsm`] macro instead.
///
/// # Examples
///
/// **Through [`prsm`] (recommended)**
///
/// ```rust
/// use prsm::{prsm_script, Script};
///
/// fn foo() -> Result<(), &'static str> { Err("this function failed!") }
/// let script: Script = prsm_script!("This is a test function", foo());
/// ```
///
/// **Manual Construction (not recommended)**
///
/// ```rust
/// use prsm::{Script, PrsmDisplay};
///
/// fn foo() -> Result<(), &'static str> { Err("this function failed!") }
/// let script: Script = Script::new(
///     "This is a test function",
///     Box::new(move || foo().map_err(|e| Box::new(e) as Box<dyn PrsmDisplay>)),
/// );
/// ```
///
/// Note that despite [`Script::new`] requiring a function with an empty parameter list, this is
/// *only* for the boxed closure that wraps around the function. In other words, your scripts *can
/// take arguments*, and their function calls are deferred until they are run using the
/// [`run`](Script::run) method.
///
/// **Function Parameters**
///
/// ```rust
/// use prsm::{prsm_script, Script};
///
/// fn parse_check(n_str: &str) -> Result<(), &'static str> {
///     if n_str.chars().any(|c| !c.is_numeric()) {
///         return Err("cannot be parsed");
///     }
///
///     Ok(())
/// }
///
/// let script: Script = prsm_script!("See if num can be parsed", parse_check("123s"));
/// let err = script.run().unwrap_err();
/// assert_eq!(format!("{}", err), "cannot be parsed");
/// ```
pub struct Script<'a> {
    /// The script description that will appear in the [`ScriptManager`] menu dialog.
    pub description: &'a str,
    func: Box<dyn Fn() -> ScriptResult>,
}

impl<'a> Script<'a> {
    /// Construct a named script that returns a [`ScriptResult`]. Manually creating a script is
    /// ill-advised since misuse could lead to a loss of data in the return. Unless you
    /// specifically need an individual script instance, consider using the [`prsm`] macro instead.
    pub fn new(description: &'a str, func: Box<dyn Fn() -> ScriptResult>) -> Self {
        Script { description, func }
    }

    /// Run the script. The error type will be morphed into a displayable item rather than the
    /// original type that was provided when creating the script instance.
    pub fn run(&self) -> ScriptResult {
        (self.func)()
    }
}

/// A named script manager (defaults to "ScriptManager"). When ran, the manager displays a mapping
/// of functions that can be called to perform tasks. The scripts can be given descriptions that
/// will display in the run menu along with their option ID. If you wish to include a script that
/// has no error condition, rather than void the usage of the `Result` type, please use
/// [`Infallible`](::std::convert::Infallible).
///
/// Although you can manually create a [`ScriptManager`] instance using [`ScriptManager::new`],
/// consider using the [`prsm`] macro instead.
///
/// # Examples
///
/// **Using [`prsm`] (recommended)**
///
/// ```rust
/// use std::convert::Infallible;
/// use prsm::{prsm, ScriptManager};
///
/// fn format() -> Result<(), std::io::Error> { Ok(()) }
/// fn lint() -> Result<(), std::io::Error> { Ok(()) }
/// fn never_fail() -> Result<(), Infallible> { Ok(()) }
///
/// let default_sm: ScriptManager = prsm! {
///     [1] "Format repository files" => format(),
///     [2] "Lint Rust files" => lint(),
///     [3] "Test never fail" => never_fail()
/// };
///
/// let named_sm: ScriptManager = prsm! {
///     CustomManager {
///         [1] "Format repository files" => format(),
///         [2] "Lint Rust files" => lint()
///     }
/// };
///
/// assert_eq!(default_sm.name, "ScriptManager");
/// assert_eq!(named_sm.name, "CustomManager");
/// ```
///
/// **Manual Construction (not recommended)**
///
/// ```rust
/// use std::collections::BTreeMap;
/// use prsm::{prsm_script, ScriptManager};
///
/// fn format() -> Result<(), std::io::Error> { Ok(()) }
/// fn lint() -> Result<(), std::io::Error> { Ok(()) }
///
/// let manual_sm: ScriptManager = ScriptManager::new(
///     Some("ManualManager"),
///     BTreeMap::from_iter([
///         (1, prsm_script!("Format repository files", format())),
///         (2, prsm_script!("Lint Rust files", lint())),
///     ]),
/// );
///
/// assert_eq!(manual_sm.name, "ManualManager");
/// ```
///
/// # Running the [`ScriptManager`]
/// You can run the [`ScriptManager`] by calling the [`run`](ScriptManager::run) function. This will
/// generate an interactive menu that accepts user input to run a script that is loaded into the
/// manager.
///
/// ```rust,no_run
/// use std::convert::Infallible;
/// use prsm::prsm;
///
/// fn format() -> Result<(), std::io::Error> { Ok(()) }
/// fn lint() -> Result<(), std::io::Error> { Ok(()) }
/// fn never_fail() -> Result<(), Infallible> { Ok(()) }
///
/// let script_manager = prsm! {
///     [1] "Format repository files" => format(),
///     [2] "Lint Rust files" => lint(),
///     [3] "Test never fail" => never_fail()
/// };
///
/// script_manager.run();
/// ```  
pub struct ScriptManager<'a> {
    /// The name of the script manager that's displayed when [`run`](ScriptManager::run) is called.
    pub name: &'a str,

    scripts: BTreeMap<usize, Script<'a>>,
}

impl<'a> Display for ScriptManager<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = format!("========== {} ==========", self.name);
        let footer = "=".repeat(header.len());

        writeln!(f, "{}", header)?;
        for (idx, script) in &self.scripts {
            writeln!(f, "[{}] {}", idx, script.description)?;
        }
        write!(f, "{}", footer)?;

        Ok(())
    }
}

impl<'a> ScriptManager<'a> {
    /// Construct a [`ScriptManager`] with the given name and scripts. The indices of the scripts
    /// in the map correlate to their option IDs that will be displayed when [`run`](ScriptManager::run)
    /// is called. If `None` is provided for `name`, then the default named "ScriptManager" is used.
    ///
    /// Although you can manually create a [`ScriptManager`] instance using this function, consider
    /// using the [`prsm`] macro instead.
    pub fn new(name: Option<&'a str>, scripts: BTreeMap<usize, Script<'a>>) -> Self {
        Self {
            name: match name {
                Some(n) => n,
                None => "ScriptManager",
            },
            scripts,
        }
    }

    /// Load the script manager's menu and then request the user for a script to be run. Any
    /// errors in the chosen script are collected as a string and returned from this function
    /// for logging.
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
        self.scripts[&opt].run().map_err(|e| format!("{}", e))
    }

    #[cfg(test)]
    fn run_script(&self, idx: usize) -> Result<(), String> {
        self.scripts[&idx].run().map_err(|e| format!("{}", e))
    }
}

/// Generates a [`Script`].
///
/// The functions provided to this macro have their calls deferred by boxing the functions into a
/// move closure. This means that even though in the macro you are "calling" the function (using
/// `()`), the function will not actually execute until the script's [`run`](Script::run) function
/// is called.
///
/// The primary focus of `prsm` is to simplify the setup of the script manager. It assumes that each script,
/// function, suite, etc. that it runs will have a high-level entry point that does not need to return
/// any value other than an error. For the sake of simplicty and ease of use, this macro *will accept*
/// functions that return results with non-unit `Ok` values. However, these values will be immediately
/// dropped and will not be accesible through the [`run`](Script::run) function nor through the
/// [`ScriptManager`].
///
/// # Example
/// ```rust
/// use prsm::{prsm_script, Script};
///
/// fn foo() -> Result<(), &'static str> { Err("this function failed!") }
/// let script: Script = prsm_script!("This is a test function", foo());
/// let err = script.run().unwrap_err();
///
/// assert_eq!(script.description, "This is a test function");
/// assert_eq!(format!("{}", err), "this function failed!");
/// ```
///
/// This macro also supports constructing scripts that take various parameters (including generics).
///
/// ```rust
/// use std::fmt::Display;
/// use prsm::prsm_script;
///
/// fn reflect<T: Display>(msg: T) -> Result<(), T> {
///     Err(msg)
/// }
///
/// let script = prsm_script!("reflection script", reflect("reflect back"));
/// let err = script.run().unwrap_err();
///
/// assert_eq!(script.description, "reflection script");
/// assert_eq!(format!("{}", err), "reflect back");
/// ```
///
/// See how non-unit return values are dropped.
///
/// ```rust
/// use prsm::prsm_script;
///
/// fn foo() -> Result<usize, &'static str> { Ok(200) }
/// let script = prsm_script!("This is a test function", foo());
/// let ok = script.run().unwrap();
///
/// assert_eq!(script.description, "This is a test function");
/// assert_eq!(ok, ());  // Uh oh... where did my 200 response go?!
/// ```
#[macro_export]
macro_rules! prsm_script {
    ($desc:literal, $f:expr) => {
        $crate::Script::new(
            $desc,
            Box::new(move || {
                $f.map(|_| ())
                    .map_err(|e| Box::new(e) as Box<dyn $crate::PrsmDisplay>)
            }),
        )
    };
}

/// Generates a [`ScriptManager`].
///
/// This macro creates script managers with script IDs and descriptions in a concise format.
/// The resulting script manager can display a menu for all of the scripts with their descriptions
/// and prompt the user for which script to run. Each ID and description are expected to be
/// compile-time literals. Functions provided to this macro are redirected through [`prsm_script`]
/// when creating the underlying script objects. Please read more there to understand how scripts
/// are used and what their limitations are.
///
/// Scripts managers can be named by wrapping the script configuration with a named type in braces,
/// mimicking a configuration file-like format.
///
/// # Examples
/// ```rust
/// use std::convert::Infallible;
/// use prsm::{prsm, ScriptManager};
///
/// fn format() -> Result<(), std::io::Error> { Ok(()) }
/// fn lint() -> Result<(), std::io::Error> { Ok(()) }
/// fn never_fail() -> Result<(), Infallible> { Ok(()) }
///
/// let default_sm: ScriptManager = prsm! {
///     [1] "Format repository files" => format(),
///     [2] "Lint Rust files" => lint(),
///     [3] "Test never fail" => never_fail()
/// };
///
/// let named_sm: ScriptManager = prsm! {
///     CustomManager {
///         [1] "Format repository files" => format(),
///         [2] "Lint Rust files" => lint()
///     }
/// };
///
/// assert_eq!(default_sm.name, "ScriptManager");
/// assert_eq!(named_sm.name, "CustomManager");
/// ```
///
/// Note the borrow checker will prevent erroneous use of stateful functions.
///
/// ```rust,compile_fail
/// # use prsm::prsm;
/// struct Foo;
/// impl Foo {
///     fn mut_1(&mut self) -> Result<(), &'static str> { Ok(()) }
///     fn mut_2(&mut self) -> Result<(), usize> { Ok(()) }
/// }
///
/// let mut foo = Foo;
/// let default_sm = prsm! {
///     [1] "Run func 1" => foo.mut_1(),
///     [2] "Run func 2" => foo.mut_2() // Cannot have multiple mutable references!
/// };
/// ```
#[macro_export]
macro_rules! prsm {
    ($([$idx:literal] $desc:literal => $f:expr),*) => {
        $crate::ScriptManager::new(None, [$(($idx, $crate::prsm_script!($desc, $f))),*]
            .into_iter()
            .collect::<::std::collections::BTreeMap<usize, $crate::Script>>()
        )
    };

    ($manager_name:ident { $([$idx:literal] $desc:literal => $f:expr),* }) => {
        $crate::ScriptManager::new(Some(stringify!($manager_name)), [$(($idx, $crate::prsm_script!($desc, $f))),*]
            .into_iter()
            .collect::<::std::collections::BTreeMap<usize, $crate::Script>>()
        )
    };
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

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

    #[test]
    fn prsm_script_basic() {
        fn foo() -> Result<(), &'static str> {
            Err("failed")
        }

        let script = prsm_script!("test script", foo());
        let err = script.run().unwrap_err();

        assert_eq!(script.description, "test script");
        assert_eq!(format!("{}", err), "failed");
    }

    #[test]
    fn prsm_script_generic_parameter() {
        fn foo<T: Sized>(_msg: T) -> Result<(), &'static str> {
            Err("failed")
        }

        let script_1 = prsm_script!("test script", foo(&[] as &[()]));
        let script_2 = prsm_script!("test script", foo(&[1, 2, 3]));

        for script in [script_1, script_2] {
            let err = script.run().unwrap_err();
            assert_eq!(script.description, "test script");
            assert_eq!(format!("{}", err), "failed");
        }
    }

    #[test]
    fn prsm_script_generic_return() {
        fn reflect<T: Display>(msg: T) -> Result<(), T> {
            Err(msg)
        }

        let script = prsm_script!("test script", reflect("reflect back"));
        let err = script.run().unwrap_err();

        assert_eq!(script.description, "test script");
        assert_eq!(format!("{}", err), "reflect back");
    }

    #[test]
    fn prsm_infallible_return() {
        fn cannot_fail() -> Result<(), Infallible> {
            Ok(())
        }

        let script = prsm_script!("infallibe", cannot_fail());
        let ok = script.run().unwrap();

        assert!(matches!(ok, ()));
    }
}

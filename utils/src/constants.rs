#[cfg(target_os = "windows")]
pub const PRSM_ROOT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "\\..\\");

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const PRSM_ROOT: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prsm_root() {
        #[cfg(target_os = "windows")]
        assert!(PRSM_ROOT.ends_with("prsm\\utils\\..\\"));
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        assert!(PRSM_ROOT.ends_with("prsm/utils/../"));
    }
}

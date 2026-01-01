#[cfg(test)]
mod tests {
    #[test]
    fn new_hermes_valid_path() {
        use hermes::Hermes;
        let hermes = Hermes::new("tmp/hermes");
        assert!(hermes.is_ok());
        assert!(std::fs::remove_dir_all("tmp/hermes").is_ok());
    }

    #[test]
    fn new_hermes_empty_path() {
        use hermes::Hermes;
        let hermes = Hermes::new("");
        assert!(hermes.is_err());
    }

    #[test]
    fn new_hermes_not_directory() {
        use hermes::Hermes;
        assert!(std::fs::File::create("tmp.data").is_ok());
        let hermes = Hermes::new("tmp.data");
        assert!(hermes.is_err());
        assert!(std::fs::remove_file("tmp.data").is_ok());
    }
}

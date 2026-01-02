#[cfg(test)]
mod connection {
    use hermes::Hermes;
    #[test]
    fn new_hermes_valid_path() {
        let hermes = Hermes::new("tmp/hermes");
        assert!(hermes.is_ok());
        assert!(std::fs::remove_dir_all("tmp").is_ok());
    }

    #[test]
    fn new_hermes_empty_path() {
        let hermes = Hermes::new("");
        assert!(hermes.is_err());
    }

    #[test]
    fn new_hermes_not_directory() {
        assert!(std::fs::File::create("tmp.data").is_ok());
        let hermes = Hermes::new("tmp.data");
        assert!(hermes.is_err());
        assert!(std::fs::remove_file("tmp.data").is_ok());
    }

    #[test]
    #[ignore = "default timeout is 197 sec"]
    fn default_timeout() {
        let t = "tmp1";
        let hermes = Hermes::new(t).unwrap();
        assert!(hermes.await_request().is_err());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }
}

#[cfg(test)]
mod req_res {
    use hermes::Hermes;
    use nabu::XffValue;
    #[test]
    fn general_request() {
        let t = "tmp2";
        let hermes = Hermes::new(t).unwrap();
        let value: XffValue = "".into();
        assert!(hermes.request(value).is_ok());
        let answer = hermes.await_request();
        assert!(answer.is_ok());
        let answer = answer.unwrap();
        assert_eq!(answer, "".into());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }

    #[test]
    fn req_res_simple() {
        let t = "tmp3";
        let hermes = Hermes::new(t).unwrap();
        let value: XffValue = "Hello".into();
        assert!(hermes.request(value).is_ok());
        let req = hermes.await_request();
        assert!(req.is_ok());
        let req = req.unwrap();
        let answer = format!("{} World!", req);
        assert!(hermes.respond(answer.into()).is_ok());
        let answer = hermes.await_response();
        assert!(answer.is_ok());
        let answer = answer.unwrap();
        assert_eq!(answer, "Hello World!".into());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }
}

#[cfg(test)]
mod errors {
    use hermes::Hermes;
    use nabu::XffValue;
    #[test]
    fn basic_error_propagation() {
        let t = "tmp4";
        let hermes = Hermes::new(t).unwrap();
        let value: XffValue = "".into();
        assert!(hermes.request(value).is_ok());
        assert!(hermes.put_error("error".into()).is_ok());
        let answer = hermes.await_request();
        println!("{:?}", answer);
        let err = answer.unwrap_err();
        let inner_err = err.get_inner_server_error();
        assert!(inner_err.is_some());
        assert_eq!(inner_err.unwrap(), XffValue::from("error"));
        assert!(std::fs::remove_dir_all(t).is_ok());
    }
}

#[cfg(test)]
/// Add bugs for debugging, keep tests for regressions
mod bugs {
    use hermes::Hermes;
    use nabu::XffValue;
    #[test]
    fn bug1() {
        let t = "bug1";
        let hermes = Hermes::new(t).unwrap();
        let value: XffValue = "".into();
        assert!(hermes.respond(value).is_ok());
        assert!(hermes.is_response_ready());
        let res = hermes.get_response();
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "".into());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }
}

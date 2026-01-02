#[cfg(test)]
mod internal_server_test {
    use hermes::Hermes;
    use nabu::XffValue;
    #[test]
    fn internal_server_test() {
        let t = "tmp5";
        let con = Hermes::new(t).unwrap();
        let request: XffValue = "World".into();
        assert!(con.request(request).is_ok());

        // Spawn new thread to handle the request
        std::thread::spawn(move || {
            let t = "tmp5";
            let con = Hermes::new(t).unwrap();
            let request = con.await_request();
            assert!(request.is_ok());
            let response: XffValue = format!("Hello {}!", request.unwrap()).into();
            assert!(con.respond(response).is_ok());
        });

        let response = con.await_response();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!".into());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }
}

#[cfg(test)]
mod several_requests {
    use hermes::Hermes;
    use nabu::XffValue;
    #[test]
    fn several_requests_client() {
        let t = "tmp8";
        let con = Hermes::new(t).unwrap();
        for i in 0..4500 {
            let request: XffValue = format!("World {}", i).into();
            assert!(con.request(request.clone()).is_ok());
            let answer = con.await_response();
            assert!(answer.is_ok());
            assert_eq!(answer.unwrap(), format!("Hello {}!", request.into_string().unwrap()).into());
        }
        assert!(std::fs::remove_dir_all(t).is_ok());
    }

    #[test]
    fn several_requests_server() {
        let t = "tmp8";
        let con = Hermes::new(t).unwrap();
        let mut i = 0;
        loop {
            let request = con.await_request();
            assert!(request.is_ok());
            let response: XffValue = format!("Hello {}!", request.unwrap()).into();
            assert!(con.respond(response).is_ok());
            i += 1;
            if i == 4500 {
                break;
            }
        }
    }
}

#[cfg(test)]
mod external_server_test {
    use hermes::Hermes;
    use nabu::{xff, XffValue};
    #[test]
    fn combined_test_partner1() {
        let t = "tmp6";
        let con = Hermes::new(t).unwrap();
        let request: XffValue = "World".into();
        assert!(con.request(request).is_ok());
        let response = con.await_response();
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), "Hello World!".into());
        assert!(std::fs::remove_dir_all(t).is_ok());
    }

    #[test]
    fn combined_test_partner2() {
        let t = "tmp6";
        let con = Hermes::new(t).unwrap();
        let request = con.await_request();
        assert!(request.is_ok());
        let response: XffValue = format!("Hello {}!", request.unwrap()).into();
        assert!(con.respond(response).is_ok());
    }

    #[test]
    fn long_running_combined_test_partner1() {
        let t = "tmp7";
        let con = Hermes::new(t).unwrap();
        let request: XffValue = xff!(vec![Into::<XffValue>::into("World"), Into::<XffValue>::into("Terminal"), Into::<XffValue>::into("Europa"), Into::<XffValue>::into("London"), Into::<XffValue>::into("Lena"), ]);
        assert!(con.request(request).is_ok());
        let response = con.await_response();
        assert!(response.is_ok());
        let expected = xff!(vec![Into::<XffValue>::into("Hello World!"), Into::<XffValue>::into("Hello Terminal!"), Into::<XffValue>::into("Hello Europa!"), Into::<XffValue>::into("Hello London!"), Into::<XffValue>::into("Hello Lena!"), ]);
        assert_eq!(response.unwrap(), expected);
        assert!(std::fs::remove_dir_all(t).is_ok());
    }

    #[test]
    fn long_running_combined_test_partner2() {
        let t = "tmp7";
        let con = Hermes::new(t).unwrap();
        let request = con.await_request();
        assert!(request.is_ok());
        let response: XffValue = {
            let mut out: Vec<XffValue> = Vec::new();
            for entry in request.unwrap().into_array().unwrap() {
                out.push(format!("Hello {}!", entry).into());
                // simulate some work
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            xff!(out)
        };
        assert!(con.respond(response).is_ok());
    }
}
